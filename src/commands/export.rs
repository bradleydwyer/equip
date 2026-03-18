use std::path::Path;

use crate::agents::{self, AGENTS};
use crate::config;
use crate::metadata::SkillMetadata;
use crate::ops;
use crate::output;
use crate::skill;
use crate::sync;

pub fn run(output_path: Option<&str>, json: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    // Build list of installed global skills
    let skills = scan_installed_skills(&project_root)?;

    if json {
        let entries: Vec<serde_json::Value> = skills
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "source": s.source,
                    "description": s.description,
                })
            })
            .collect();
        println!(
            "{}",
            serde_json::to_string_pretty(&entries)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
        return Ok(());
    }

    if let Some(path) = output_path {
        let entries: Vec<serde_json::Value> = skills
            .iter()
            .map(|s| {
                serde_json::json!({
                    "name": s.name,
                    "source": s.source,
                    "description": s.description,
                })
            })
            .collect();
        let json_str = serde_json::to_string_pretty(&entries)
            .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
        std::fs::write(path, json_str).map_err(|e| format!("Failed to write {path}: {e}"))?;
        println!(
            "{} Exported {} skill(s) to {}",
            output::green("✓"),
            skills.len(),
            path
        );
        return Ok(());
    }

    // Backend mode — write ops and copy skill content
    let cfg = config::read()?.ok_or_else(|| {
        "No sync backend configured. Run 'equip init' first, or use --output for file export."
            .to_string()
    })?;

    let ops_dir = config::ops_dir(&cfg)?;
    let skills_dir = config::skills_dir(&cfg)?;
    sync::pull(&cfg)?;

    let mut exported = 0;

    for skill in &skills {
        // Always write an op to keep the log in sync with content
        let op = ops::add_op(&skill.name, skill.source.as_deref(), &skill.description);
        ops::write_op(&ops_dir, &op)?;

        // Always copy/update skill content
        let dest = skills_dir.join(&skill.name);
        copy_skill_dir(&skill.path, &dest)?;
        exported += 1;
    }

    if exported > 0 {
        sync::push(&cfg)?;
        println!("{} Exported {} skill(s)", output::green("✓"), exported,);
    } else {
        println!("{} No skills to export.", output::green("✓"),);
    }

    Ok(())
}

struct InstalledSkill {
    name: String,
    source: Option<String>,
    description: String,
    path: std::path::PathBuf,
}

fn scan_installed_skills(project_root: &std::path::Path) -> Result<Vec<InstalledSkill>, String> {
    let mut seen = std::collections::BTreeMap::new();

    for agent in AGENTS {
        let dir = agents::skill_dir(agent, true, project_root)?;
        if !dir.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() || !path.join("SKILL.md").exists() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            if seen.contains_key(&name) {
                continue;
            }
            let description = skill::read_skill(&path)
                .map(|fm| fm.description)
                .unwrap_or_default();
            let source = SkillMetadata::read(&path).ok().map(|m| m.source);
            seen.insert(
                name.clone(),
                InstalledSkill {
                    name,
                    source,
                    description,
                    path: path.clone(),
                },
            );
        }
    }

    Ok(seen.into_values().collect())
}

/// Copy a skill directory to a destination, excluding .git and .equip.json
fn copy_skill_dir(src: &Path, dest: &Path) -> Result<(), String> {
    // Remove existing destination to get a clean copy
    if dest.exists() {
        std::fs::remove_dir_all(dest)
            .map_err(|e| format!("Failed to clean {}: {e}", dest.display()))?;
    }
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create {}: {e}", dest.display()))?;
    copy_dir_recursive(src, dest)
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let src_path = entry.path();
        let file_name = entry.file_name();

        let name = file_name.to_string_lossy();
        if name == ".git" || name == ".equip.json" {
            continue;
        }

        let dest_path = dest.join(&file_name);
        if src_path.is_dir() {
            std::fs::create_dir_all(&dest_path)
                .map_err(|e| format!("Failed to create {}: {e}", dest_path.display()))?;
            copy_dir_recursive(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {}: {e}", src_path.display()))?;
        }
    }
    Ok(())
}
