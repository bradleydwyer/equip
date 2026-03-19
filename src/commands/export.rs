use crate::agents::{self, AGENTS};
use crate::config;
use crate::ops;
use crate::output;
use crate::registry;
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

    // Backend mode — write ops only (source repos are the source of truth)
    let cfg = config::read()?.ok_or_else(|| {
        "No sync backend configured. Run 'equip init' first, or use --output for file export."
            .to_string()
    })?;

    let ops_dir = config::ops_dir(&cfg)?;
    sync::pull(&cfg)?;

    let mut exported = 0;

    for skill in &skills {
        let op = ops::add_op(&skill.name, skill.source.as_deref(), &skill.description);
        ops::write_op(&ops_dir, &op)?;
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
}

fn scan_installed_skills(project_root: &std::path::Path) -> Result<Vec<InstalledSkill>, String> {
    let reg = registry::Registry::load()?;
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
            let source = reg
                .get(registry::scope_global(), &name)
                .map(|e| e.source.clone());
            seen.insert(
                name.clone(),
                InstalledSkill {
                    name,
                    source,
                    description,
                },
            );
        }
    }

    Ok(seen.into_values().collect())
}
