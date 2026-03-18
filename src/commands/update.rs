use crate::agents::{self, AGENTS};
use crate::hash;
use crate::metadata::SkillMetadata;
use crate::output;

pub fn run(name: Option<&str>, global: bool, json: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let mut to_update: Vec<(String, std::path::PathBuf, SkillMetadata)> = Vec::new();

    for agent in AGENTS {
        let dir = agents::skill_dir(agent, global, &project_root)?;
        if !dir.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let skill_name = entry.file_name().to_string_lossy().to_string();

            if let Some(target) = name
                && skill_name != target
            {
                continue;
            }

            if to_update.iter().any(|(n, _, _)| n == &skill_name) {
                continue;
            }

            if let Ok(meta) = SkillMetadata::read(&path) {
                to_update.push((skill_name, path, meta));
            }
        }
    }

    if to_update.is_empty() {
        if let Some(target) = name {
            return Err(format!(
                "Skill '{}' not found or has no metadata. Run 'equip list' to see installed skills.",
                target
            ));
        }
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "update",
                    "updated": [],
                }))
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            );
        } else {
            println!("No skills with metadata found to update.");
        }
        return Ok(());
    }

    if !json {
        println!("Updating {} skill(s)...\n", to_update.len());
    }

    let mut results: Vec<serde_json::Value> = Vec::new();

    for (skill_name, skill_path, meta) in &to_update {
        // Skip adopted skills — they have no real source to update from
        if meta.source == "adopted" {
            if !json {
                println!(
                    "  {} {} {}",
                    output::dim("·"),
                    output::bold(skill_name),
                    output::dim("adopted (skipped)")
                );
            }
            results.push(serde_json::json!({
                "name": skill_name,
                "source": meta.source,
                "status": "skipped",
                "reason": "adopted",
            }));
            continue;
        }

        if !json {
            // Check for local modifications before overwriting
            if let Some(installed_hash) = &meta.content_hash {
                let current = format!("{:016x}", hash::hash_skill_dir(skill_path));
                if &current != installed_hash {
                    println!(
                        "  {} {} has local changes that will be overwritten",
                        output::yellow("!"),
                        output::bold(skill_name)
                    );
                }
            }

            print!("  {} ", output::bold(skill_name));
        }

        let agent_ids = meta.agents.clone();
        // Pass json=false to install so it doesn't double-print JSON
        match super::install::run(&meta.source, global, &agent_ids, false, false) {
            Ok(()) => {
                results.push(serde_json::json!({
                    "name": skill_name,
                    "source": meta.source,
                    "status": "updated",
                }));
            }
            Err(e) => {
                if !json {
                    eprintln!("{}", output::red(&format!("  ✗ {e}")));
                }
                results.push(serde_json::json!({
                    "name": skill_name,
                    "source": meta.source,
                    "status": "failed",
                    "error": e,
                }));
            }
        }
    }

    if json {
        let out = serde_json::json!({
            "action": "update",
            "global": global,
            "updated": results,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
    }

    Ok(())
}
