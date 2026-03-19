use std::collections::BTreeSet;

use crate::agents::{self, AGENTS};
use crate::config;
use crate::ops;
use crate::output;
use crate::registry;
use crate::sync;

pub fn run(json: bool) -> Result<(), String> {
    let cfg = match config::read()? {
        Some(c) => c,
        None => {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "error": "No sync backend configured",
                    }))
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?
                );
            } else {
                println!(
                    "No sync backend configured. Run {} to set one up.",
                    output::bold("equip init")
                );
            }
            return Ok(());
        }
    };

    sync::pull(&cfg)?;

    let ops_dir = config::ops_dir(&cfg)?;
    let manifest_state = ops::compute_state(&ops_dir)?;

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    // Scan installed global skills
    let reg = registry::Registry::load()?;
    let mut installed: std::collections::BTreeMap<String, Option<String>> =
        std::collections::BTreeMap::new();
    for agent in AGENTS {
        let dir = agents::skill_dir(agent, true, &project_root)?;
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
            installed.entry(name.clone()).or_insert_with(|| {
                reg.get(registry::scope_global(), &name)
                    .map(|e| e.source.clone())
            });
        }
    }

    let manifest_names: BTreeSet<&str> = manifest_state.keys().map(|s| s.as_str()).collect();
    let installed_names: BTreeSet<&str> = installed.keys().map(|s| s.as_str()).collect();

    let synced: Vec<&str> = manifest_names
        .intersection(&installed_names)
        .copied()
        .collect();
    let missing: Vec<&str> = manifest_names
        .difference(&installed_names)
        .copied()
        .collect();
    let untracked: Vec<&str> = installed_names
        .difference(&manifest_names)
        .copied()
        .collect();

    if json {
        let out = serde_json::json!({
            "synced": synced,
            "missing": missing,
            "untracked": untracked,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
    } else {
        if !synced.is_empty() {
            for name in &synced {
                println!("  {} {}", output::green("✓"), name);
            }
        }
        if !missing.is_empty() {
            for name in &missing {
                let source = manifest_state
                    .get(*name)
                    .and_then(|s| s.source.as_deref())
                    .unwrap_or("unknown source");
                println!(
                    "  {} {} {}",
                    output::red("✗"),
                    output::bold(name),
                    output::dim(&format!("({})", source))
                );
            }
        }
        if !untracked.is_empty() {
            for name in &untracked {
                println!(
                    "  {} {} {}",
                    output::yellow("?"),
                    output::bold(name),
                    output::dim("(untracked)")
                );
            }
        }

        println!(
            "\n{} synced, {} missing, {} untracked",
            synced.len(),
            missing.len(),
            untracked.len()
        );
    }

    Ok(())
}
