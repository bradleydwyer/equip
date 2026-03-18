use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

use crate::metadata;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OpKind {
    Add,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Op {
    pub op: OpKind,
    pub skill: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub ts: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct SkillState {
    pub source: Option<String>,
    pub description: String,
}

/// Validate a skill name is safe for use in filenames and paths
fn validate_skill_name(name: &str) -> Result<(), String> {
    if name.is_empty()
        || name.contains('/')
        || name.contains('\\')
        || name == ".."
        || name == "."
        || name.contains("..")
    {
        return Err(format!("Invalid skill name in op: '{name}'"));
    }
    Ok(())
}

/// Write a new op file to the ops directory.
/// Filename: {timestamp_ms}-{op}-{skill}.json
pub fn write_op(ops_dir: &Path, op: &Op) -> Result<(), String> {
    validate_skill_name(&op.skill)?;

    std::fs::create_dir_all(ops_dir)
        .map_err(|e| format!("Failed to create {}: {e}", ops_dir.display()))?;

    let ts_safe = op.ts.replace([':', '-'], "");
    let op_str = match op.op {
        OpKind::Add => "add",
        OpKind::Remove => "remove",
    };
    // Include millis from system clock for uniqueness within the same second
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        % 1000;
    let filename = format!("{}-{:03}-{}-{}.json", ts_safe, millis, op_str, op.skill);
    let path = ops_dir.join(&filename);

    let json =
        serde_json::to_string_pretty(op).map_err(|e| format!("Failed to serialize op: {e}"))?;
    std::fs::write(&path, json).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

/// Read all op files from a directory, compute the current state.
/// Returns only active skills (latest op is "add").
pub fn compute_state(ops_dir: &Path) -> Result<BTreeMap<String, SkillState>, String> {
    if !ops_dir.exists() {
        return Ok(BTreeMap::new());
    }

    let mut all_ops: Vec<Op> = Vec::new();

    let entries = std::fs::read_dir(ops_dir)
        .map_err(|e| format!("Failed to read {}: {e}", ops_dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        if let Ok(op) = serde_json::from_str::<Op>(&content) {
            // Skip ops with invalid skill names (path traversal protection)
            if validate_skill_name(&op.skill).is_ok() {
                all_ops.push(op);
            }
        }
    }

    // Sort by timestamp
    all_ops.sort_by(|a, b| a.ts.cmp(&b.ts));

    // Group by skill, take latest op
    let mut latest: BTreeMap<String, Op> = BTreeMap::new();
    for op in all_ops {
        latest.insert(op.skill.clone(), op);
    }

    // Filter to active skills only
    let mut state = BTreeMap::new();
    for (name, op) in latest {
        if op.op == OpKind::Add {
            state.insert(
                name,
                SkillState {
                    source: op.source,
                    description: op.description.unwrap_or_default(),
                },
            );
        }
    }

    Ok(state)
}

/// Create an "add" op for a skill
pub fn add_op(skill: &str, source: Option<&str>, description: &str) -> Op {
    Op {
        op: OpKind::Add,
        skill: skill.to_string(),
        source: source.map(String::from),
        description: Some(description.to_string()),
        ts: metadata::now_iso8601(),
    }
}

/// Create a "remove" op for a skill
pub fn remove_op(skill: &str) -> Op {
    Op {
        op: OpKind::Remove,
        skill: skill.to_string(),
        source: None,
        description: None,
        ts: metadata::now_iso8601(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_add(skill: &str, source: Option<&str>, description: &str, ts: &str) -> Op {
        Op {
            op: OpKind::Add,
            skill: skill.to_string(),
            source: source.map(String::from),
            description: Some(description.to_string()),
            ts: ts.to_string(),
        }
    }

    fn make_remove(skill: &str, ts: &str) -> Op {
        Op {
            op: OpKind::Remove,
            skill: skill.to_string(),
            source: None,
            description: None,
            ts: ts.to_string(),
        }
    }

    #[test]
    fn ops_write_creates_file() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");
        let op = make_add(
            "my-skill",
            Some("owner/repo"),
            "A test skill",
            "2026-03-15T10:00:00Z",
        );

        write_op(&ops_dir, &op).unwrap();

        let entries: Vec<_> = std::fs::read_dir(&ops_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("json"))
            .collect();
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn ops_compute_state_single_add() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");
        let op = make_add(
            "my-skill",
            Some("owner/repo"),
            "A test skill",
            "2026-03-15T10:00:00Z",
        );

        write_op(&ops_dir, &op).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert_eq!(state.len(), 1);
        assert!(state.contains_key("my-skill"));
        assert_eq!(state["my-skill"].source.as_deref(), Some("owner/repo"));
        assert_eq!(state["my-skill"].description, "A test skill");
    }

    #[test]
    fn ops_compute_state_add_then_remove() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");

        let add = make_add(
            "my-skill",
            Some("owner/repo"),
            "A test skill",
            "2026-03-15T10:00:00Z",
        );
        let remove = make_remove("my-skill", "2026-03-16T10:00:00Z");

        write_op(&ops_dir, &add).unwrap();
        write_op(&ops_dir, &remove).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert!(state.is_empty());
    }

    #[test]
    fn ops_compute_state_remove_then_readd() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");

        let add1 = make_add(
            "my-skill",
            Some("owner/repo"),
            "First add",
            "2026-03-15T10:00:00Z",
        );
        let remove = make_remove("my-skill", "2026-03-16T10:00:00Z");
        let add2 = make_add(
            "my-skill",
            Some("other/repo"),
            "Re-added",
            "2026-03-17T10:00:00Z",
        );

        write_op(&ops_dir, &add1).unwrap();
        write_op(&ops_dir, &remove).unwrap();
        write_op(&ops_dir, &add2).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert_eq!(state.len(), 1);
        assert_eq!(state["my-skill"].source.as_deref(), Some("other/repo"));
        assert_eq!(state["my-skill"].description, "Re-added");
    }

    #[test]
    fn ops_compute_state_multiple_skills() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");

        let op1 = make_add("skill-a", Some("src-a"), "Skill A", "2026-03-15T10:00:00Z");
        let op2 = make_add("skill-b", Some("src-b"), "Skill B", "2026-03-15T11:00:00Z");
        let op3 = make_add("skill-c", Some("src-c"), "Skill C", "2026-03-15T12:00:00Z");

        write_op(&ops_dir, &op1).unwrap();
        write_op(&ops_dir, &op2).unwrap();
        write_op(&ops_dir, &op3).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert_eq!(state.len(), 3);
        assert!(state.contains_key("skill-a"));
        assert!(state.contains_key("skill-b"));
        assert!(state.contains_key("skill-c"));
    }

    #[test]
    fn ops_compute_state_latest_wins() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");

        let op1 = make_add(
            "my-skill",
            Some("first/source"),
            "First version",
            "2026-03-15T10:00:00Z",
        );
        let op2 = make_add(
            "my-skill",
            Some("second/source"),
            "Updated version",
            "2026-03-16T10:00:00Z",
        );

        write_op(&ops_dir, &op1).unwrap();
        write_op(&ops_dir, &op2).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert_eq!(state.len(), 1);
        assert_eq!(state["my-skill"].source.as_deref(), Some("second/source"));
        assert_eq!(state["my-skill"].description, "Updated version");
    }

    #[test]
    fn ops_compute_state_empty_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");
        std::fs::create_dir_all(&ops_dir).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert!(state.is_empty());
    }

    #[test]
    fn ops_compute_state_ignores_non_json() {
        let tmp = tempfile::tempdir().unwrap();
        let ops_dir = tmp.path().join("ops");
        std::fs::create_dir_all(&ops_dir).unwrap();

        // Write a .txt file that should be ignored
        std::fs::write(ops_dir.join("notes.txt"), "not an op file").unwrap();

        // Write one real op
        let op = make_add(
            "my-skill",
            Some("owner/repo"),
            "A skill",
            "2026-03-15T10:00:00Z",
        );
        write_op(&ops_dir, &op).unwrap();

        let state = compute_state(&ops_dir).unwrap();
        assert_eq!(state.len(), 1);
        assert!(state.contains_key("my-skill"));
    }

    #[test]
    fn ops_op_serialization_roundtrip() {
        let op = make_add(
            "test-skill",
            Some("owner/repo"),
            "Test description",
            "2026-03-15T10:00:00Z",
        );

        let json = serde_json::to_string_pretty(&op).unwrap();
        let parsed: Op = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.op, OpKind::Add);
        assert_eq!(parsed.skill, "test-skill");
        assert_eq!(parsed.source.as_deref(), Some("owner/repo"));
        assert_eq!(parsed.description.as_deref(), Some("Test description"));
        assert_eq!(parsed.ts, "2026-03-15T10:00:00Z");

        // Also test remove variant roundtrip
        let remove = make_remove("test-skill", "2026-03-16T10:00:00Z");
        let json = serde_json::to_string_pretty(&remove).unwrap();
        let parsed: Op = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.op, OpKind::Remove);
        assert_eq!(parsed.skill, "test-skill");
        assert!(parsed.source.is_none());
        assert!(parsed.description.is_none());
    }
}
