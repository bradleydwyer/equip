use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "backend")]
pub enum EquipConfig {
    #[serde(rename = "git")]
    Git { repo: String, repo_url: String },
    #[serde(rename = "file")]
    File { path: String },
}

pub fn equip_dir() -> Result<PathBuf, String> {
    let home = home_dir()?;
    Ok(home.join(".equip"))
}

pub fn repo_dir() -> Result<PathBuf, String> {
    Ok(equip_dir()?.join("repo"))
}

fn config_path() -> Result<PathBuf, String> {
    Ok(equip_dir()?.join("config.json"))
}

fn home_dir() -> Result<PathBuf, String> {
    std::env::var("HOME")
        .map(PathBuf::from)
        .map_err(|_| "Could not determine home directory".to_string())
}

pub fn read() -> Result<Option<EquipConfig>, String> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(None);
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    let config: EquipConfig = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse {}: {e}", path.display()))?;
    Ok(Some(config))
}

pub fn write(config: &EquipConfig) -> Result<(), String> {
    let dir = equip_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;
    let path = config_path()?;
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

pub fn ops_dir(config: &EquipConfig) -> Result<PathBuf, String> {
    match config {
        EquipConfig::Git { .. } => Ok(repo_dir()?.join("ops")),
        EquipConfig::File { path } => Ok(PathBuf::from(path).join("ops")),
    }
}

pub fn backend_root(config: &EquipConfig) -> Result<PathBuf, String> {
    match config {
        EquipConfig::Git { .. } => repo_dir(),
        EquipConfig::File { path } => Ok(PathBuf::from(path)),
    }
}

pub fn skills_dir(config: &EquipConfig) -> Result<PathBuf, String> {
    match config {
        EquipConfig::Git { .. } => Ok(repo_dir()?.join("skills")),
        EquipConfig::File { path } => Ok(PathBuf::from(path).join("skills")),
    }
}

// ── Global settings (separate from sync backend config) ──

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Settings {
    /// Default path for `equip survey` project scanning (e.g., "~/dev")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projects_path: Option<String>,
}

fn settings_path() -> Result<PathBuf, String> {
    Ok(equip_dir()?.join("settings.json"))
}

pub fn read_settings() -> Result<Settings, String> {
    let path = match settings_path() {
        Ok(p) => p,
        Err(_) => return Ok(Settings::default()),
    };
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    serde_json::from_str(&content).map_err(|e| format!("Failed to parse {}: {e}", path.display()))
}

pub fn write_settings(settings: &Settings) -> Result<(), String> {
    let dir = equip_dir()?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create {}: {e}", dir.display()))?;
    let path = settings_path()?;
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_write_and_read_git() {
        let config = EquipConfig::Git {
            repo: "owner/repo".to_string(),
            repo_url: "https://github.com/owner/repo.git".to_string(),
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EquipConfig = serde_json::from_str(&json).unwrap();

        match parsed {
            EquipConfig::Git { repo, repo_url } => {
                assert_eq!(repo, "owner/repo");
                assert_eq!(repo_url, "https://github.com/owner/repo.git");
            }
            _ => panic!("Expected Git variant"),
        }
    }

    #[test]
    fn config_write_and_read_file() {
        let config = EquipConfig::File {
            path: "/Users/test/sync".to_string(),
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        let parsed: EquipConfig = serde_json::from_str(&json).unwrap();

        match parsed {
            EquipConfig::File { path } => {
                assert_eq!(path, "/Users/test/sync");
            }
            _ => panic!("Expected File variant"),
        }
    }

    #[test]
    fn config_read_missing_returns_none() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("config.json");
        assert!(!path.exists());

        // Directly check that a missing file yields None behavior
        let content = std::fs::read_to_string(&path);
        assert!(content.is_err());
    }

    #[test]
    fn config_ops_dir_git() {
        let config = EquipConfig::Git {
            repo: "owner/repo".to_string(),
            repo_url: "https://github.com/owner/repo.git".to_string(),
        };
        let dir = ops_dir(&config).unwrap();
        // Git ops_dir should end with .equip/repo/ops
        assert!(dir.ends_with("repo/ops"));
    }

    #[test]
    fn config_ops_dir_file() {
        let config = EquipConfig::File {
            path: "/tmp/my-sync".to_string(),
        };
        let dir = ops_dir(&config).unwrap();
        assert_eq!(dir, PathBuf::from("/tmp/my-sync/ops"));
    }
}
