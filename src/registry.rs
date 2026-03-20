use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::config;
use crate::metadata::SkillMetadata;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    pub skill_name: String,
    pub scope: String,
    pub source: String,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subpath: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    pub installed_at: String,
    pub agents: Vec<String>,
    pub equip_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_date: Option<String>,
}

impl RegistryEntry {
    /// Convert to SkillMetadata (minimises changes in update.rs/outdated.rs which use it extensively).
    pub fn as_metadata(&self) -> SkillMetadata {
        SkillMetadata {
            source: self.source.clone(),
            source_type: self.source_type.clone(),
            repo_url: self.repo_url.clone(),
            subpath: self.subpath.clone(),
            local_path: self.local_path.clone(),
            installed_at: self.installed_at.clone(),
            agents: self.agents.clone(),
            equip_version: self.equip_version.clone(),
            source_commit: self.source_commit.clone(),
            content_hash: self.content_hash.clone(),
            version: self.version.clone(),
            source_tag: self.source_tag.clone(),
            commit_date: self.commit_date.clone(),
            source_date: self.source_date.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Registry {
    pub version: u32,
    pub entries: BTreeMap<String, RegistryEntry>,
}

impl Registry {
    /// Load from `~/.equip/registry.json`. Returns empty registry if file is missing.
    pub fn load() -> Result<Self, String> {
        let path = registry_path()?;
        if !path.exists() {
            return Ok(Registry {
                version: 1,
                entries: BTreeMap::new(),
            });
        }
        let content = std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {e}", path.display()))
    }

    /// Atomic write: write to .tmp then rename.
    pub fn save(&self) -> Result<(), String> {
        let path = registry_path()?;
        let dir = path
            .parent()
            .ok_or_else(|| "Cannot determine registry parent dir".to_string())?;
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;

        let tmp = path.with_extension("json.tmp");
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize registry: {e}"))?;
        std::fs::write(&tmp, json)
            .map_err(|e| format!("Failed to write {}: {e}", tmp.display()))?;
        std::fs::rename(&tmp, &path).map_err(|e| {
            format!(
                "Failed to rename {} to {}: {e}",
                tmp.display(),
                path.display()
            )
        })
    }

    /// Build the key for an entry: `"{scope}/{skill_name}"`.
    pub fn entry_key(scope: &str, skill_name: &str) -> String {
        format!("{scope}/{skill_name}")
    }

    /// Insert or update an entry. Merges agent lists (union) rather than overwriting.
    pub fn upsert(&mut self, entry: RegistryEntry) {
        let key = Self::entry_key(&entry.scope, &entry.skill_name);
        if let Some(existing) = self.entries.get_mut(&key) {
            // Merge agents (union)
            for agent in &entry.agents {
                if !existing.agents.contains(agent) {
                    existing.agents.push(agent.clone());
                }
            }
            // Overwrite other fields
            existing.source = entry.source;
            existing.source_type = entry.source_type;
            existing.repo_url = entry.repo_url;
            existing.subpath = entry.subpath;
            existing.local_path = entry.local_path;
            existing.installed_at = entry.installed_at;
            existing.equip_version = entry.equip_version;
            existing.source_commit = entry.source_commit;
            existing.content_hash = entry.content_hash;
            existing.version = entry.version;
            existing.source_tag = entry.source_tag;
            existing.commit_date = entry.commit_date;
            existing.source_date = entry.source_date;
        } else {
            self.entries.insert(key, entry);
        }
    }

    /// Remove an entry entirely.
    pub fn remove_entry(&mut self, scope: &str, skill_name: &str) {
        let key = Self::entry_key(scope, skill_name);
        self.entries.remove(&key);
    }

    /// Remove specific agents from an entry. Deletes the entry if agents list becomes empty.
    pub fn remove_agents(&mut self, scope: &str, skill_name: &str, agents: &[String]) {
        let key = Self::entry_key(scope, skill_name);
        if let Some(entry) = self.entries.get_mut(&key) {
            entry.agents.retain(|a| !agents.contains(a));
            if entry.agents.is_empty() {
                self.entries.remove(&key);
            }
        }
    }

    /// Get an entry by scope and skill name.
    pub fn get(&self, scope: &str, skill_name: &str) -> Option<&RegistryEntry> {
        let key = Self::entry_key(scope, skill_name);
        self.entries.get(&key)
    }

    /// Find an entry by source string within a scope (for rename detection).
    /// Returns None if multiple entries share the same source (multi-skill repo).
    pub fn find_unique_by_source(&self, scope: &str, source: &str) -> Option<&RegistryEntry> {
        let prefix = format!("{scope}/");
        let matches: Vec<_> = self
            .entries
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .filter(|(_, v)| v.source == source)
            .collect();
        if matches.len() == 1 {
            Some(matches[0].1)
        } else {
            None
        }
    }

    /// Get all entries for a given scope.
    pub fn entries_for_scope(&self, scope: &str) -> Vec<&RegistryEntry> {
        let prefix = format!("{scope}/");
        self.entries
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix))
            .map(|(_, v)| v)
            .collect()
    }
}

pub fn scope_global() -> &'static str {
    "global"
}

pub fn scope_for_project(project_root: &Path) -> String {
    let path = project_root
        .canonicalize()
        .unwrap_or_else(|_| project_root.to_path_buf())
        .display()
        .to_string();
    // Normalize backslashes to forward slashes for cross-platform consistency
    path.replace('\\', "/")
}

fn registry_path() -> Result<PathBuf, String> {
    Ok(config::equip_dir()?.join("registry.json"))
}

/// Find the first agent dir that has this skill on disk.
pub fn find_skill_path(name: &str, global: bool, project_root: &Path) -> Option<PathBuf> {
    use crate::agents::{self, AGENTS};
    for agent in AGENTS {
        if let Ok(dir) = agents::skill_dir(agent, global, project_root) {
            let path = dir.join(name);
            if path.exists() && path.join("SKILL.md").exists() {
                return Some(path);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_entry(name: &str, scope: &str) -> RegistryEntry {
        RegistryEntry {
            skill_name: name.to_string(),
            scope: scope.to_string(),
            source: "test/source".to_string(),
            source_type: "git".to_string(),
            repo_url: None,
            subpath: None,
            local_path: None,
            installed_at: "2026-03-19T00:00:00Z".to_string(),
            agents: vec!["claude".to_string()],
            equip_version: "0.2.0".to_string(),
            source_commit: None,
            content_hash: None,
            version: None,
            source_tag: None,
            commit_date: None,
            source_date: None,
        }
    }

    #[test]
    fn entry_key_format() {
        assert_eq!(Registry::entry_key("global", "agg"), "global/agg");
        assert_eq!(
            Registry::entry_key("/Users/brad/dev", "agg"),
            "/Users/brad/dev/agg"
        );
    }

    #[test]
    fn save_and_load_roundtrip() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("registry.json");

        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        reg.upsert(sample_entry("agg", "global"));

        // Save directly to a file
        let json = serde_json::to_string_pretty(&reg).unwrap();
        std::fs::write(&path, &json).unwrap();

        // Load from the file
        let content = std::fs::read_to_string(&path).unwrap();
        let loaded: Registry = serde_json::from_str(&content).unwrap();
        assert_eq!(loaded.entries.len(), 1);
        assert!(loaded.entries.contains_key("global/agg"));
    }

    #[test]
    fn upsert_merges_agents() {
        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        reg.upsert(sample_entry("agg", "global"));

        let mut entry2 = sample_entry("agg", "global");
        entry2.agents = vec!["cursor".to_string()];
        entry2.source = "new/source".to_string();
        reg.upsert(entry2);

        let e = reg.get("global", "agg").unwrap();
        assert_eq!(e.agents, vec!["claude".to_string(), "cursor".to_string()]);
        assert_eq!(e.source, "new/source");
    }

    #[test]
    fn upsert_does_not_duplicate_agents() {
        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        reg.upsert(sample_entry("agg", "global"));
        reg.upsert(sample_entry("agg", "global"));

        let e = reg.get("global", "agg").unwrap();
        assert_eq!(e.agents, vec!["claude".to_string()]);
    }

    #[test]
    fn remove_entry_works() {
        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        reg.upsert(sample_entry("agg", "global"));
        assert!(reg.get("global", "agg").is_some());

        reg.remove_entry("global", "agg");
        assert!(reg.get("global", "agg").is_none());
    }

    #[test]
    fn remove_agents_partial() {
        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        let mut entry = sample_entry("agg", "global");
        entry.agents = vec!["claude".to_string(), "cursor".to_string()];
        reg.upsert(entry);

        reg.remove_agents("global", "agg", &["claude".to_string()]);
        let e = reg.get("global", "agg").unwrap();
        assert_eq!(e.agents, vec!["cursor".to_string()]);
    }

    #[test]
    fn remove_agents_deletes_entry_when_empty() {
        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        reg.upsert(sample_entry("agg", "global"));

        reg.remove_agents("global", "agg", &["claude".to_string()]);
        assert!(reg.get("global", "agg").is_none());
    }

    #[test]
    fn get_returns_none_for_missing() {
        let reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        assert!(reg.get("global", "nonexistent").is_none());
    }

    #[test]
    fn entries_for_scope_filters() {
        let mut reg = Registry {
            version: 1,
            entries: BTreeMap::new(),
        };
        reg.upsert(sample_entry("agg", "global"));
        reg.upsert(sample_entry("pdf", "global"));
        reg.upsert(sample_entry("agg", "/Users/brad/dev"));

        let global = reg.entries_for_scope("global");
        assert_eq!(global.len(), 2);

        let project = reg.entries_for_scope("/Users/brad/dev");
        assert_eq!(project.len(), 1);
    }
}
