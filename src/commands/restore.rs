use std::collections::HashMap;
use std::path::PathBuf;

use crate::commands::install;
use crate::config;
use crate::ops;
use crate::output;
use crate::skill;
use crate::source::SkillSource;
use crate::sync;

pub fn run(from: Option<&str>, dry_run: bool, json: bool) -> Result<(), String> {
    let skills = if let Some(path) = from {
        read_from_file(path)?
    } else {
        read_from_backend()?
    };

    if skills.is_empty() {
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "restore",
                    "restored": 0,
                    "skipped": 0,
                    "failed": 0,
                }))
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            );
        } else {
            println!("No skills to restore.");
        }
        return Ok(());
    }

    if dry_run {
        if json {
            let entries: Vec<serde_json::Value> = skills
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "name": s.name,
                        "source": s.source,
                        "has_content": s.local_path.is_some(),
                        "status": "would_install",
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "restore",
                    "dry_run": true,
                    "skills": entries,
                }))
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            );
        } else {
            println!("Dry run — would restore {} skill(s):\n", skills.len());
            for s in &skills {
                let detail = if s.local_path.is_some() {
                    "(from repo)"
                } else if s.source.is_some() {
                    s.source.as_deref().unwrap_or("")
                } else {
                    "(no source — skip)"
                };
                println!("  {} {}", output::bold(&s.name), output::dim(detail));
            }
        }
        return Ok(());
    }

    // Collect equip-includes sources (only in backend mode)
    let includes: Vec<String> = if from.is_none()
        && let Ok(Some(cfg)) = config::read()
        && let Ok(root) = config::backend_root(&cfg)
    {
        let includes_path = root.join("equip-includes");
        if includes_path.exists() {
            skill::read_includes(&includes_path).unwrap_or_default()
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    let total_count = skills.len() + includes.len();
    if !json {
        println!("Restoring {} skill(s)...\n", total_count);
    }

    // Phase 1: Pre-clone all unique remote repos in parallel
    let clones = pre_clone_repos(&skills, &includes);

    // Phase 2: Install sequentially from pre-cloned dirs (fast — no network)
    let mut restored = 0;
    let mut skipped = 0;
    let mut failed = 0;
    let mut results: Vec<serde_json::Value> = Vec::new();

    for s in &skills {
        if !json {
            print!("  {} ", output::bold(&s.name));
        }

        let result = if let Some(source) = &s.source
            && source != "adopted"
        {
            if let Some(clone_dir) = clones.get(source.as_str()) {
                install::run_from_clone(source, clone_dir, true, &[], true)
            } else {
                install::run_quiet_no_sync(source, true, &[], true)
            }
        } else if let Some(local_path) = &s.local_path {
            install::run_quiet_no_sync(&local_path.display().to_string(), true, &[], true)
        } else {
            if !json {
                println!("{}", output::dim("(no source — skipped)"));
            }
            results.push(serde_json::json!({
                "name": s.name,
                "status": "skipped",
                "reason": "no source",
            }));
            skipped += 1;
            continue;
        };

        match result {
            Ok(()) => {
                if !json {
                    println!("{}", output::green("✓"));
                }
                restored += 1;
                results.push(serde_json::json!({
                    "name": s.name,
                    "status": "restored",
                }));
            }
            Err(e) => {
                if !json {
                    eprintln!("{}", output::red(&format!("✗ {e}")));
                }
                results.push(serde_json::json!({
                    "name": s.name,
                    "status": "failed",
                    "error": e,
                }));
                failed += 1;
            }
        }
    }

    // Install includes from pre-cloned dirs
    if !includes.is_empty() {
        if !json {
            println!("\nRestoring {} include(s)...\n", includes.len());
        }
        for source in &includes {
            if !json {
                print!("  {} ", output::bold(source));
            }
            let result = if let Some(clone_dir) = clones.get(source.as_str()) {
                install::run_from_clone(source, clone_dir, true, &[], true)
            } else {
                install::run_quiet_no_sync(source, true, &[], true)
            };
            match result {
                Ok(()) => {
                    if !json {
                        println!("{}", output::green("✓"));
                    }
                    restored += 1;
                }
                Err(e) => {
                    if !json {
                        eprintln!("{}", output::red(&format!("✗ {e}")));
                    }
                    failed += 1;
                }
            }
        }
    }

    // Cleanup pre-cloned temp dirs
    for dir in clones.values() {
        let _ = std::fs::remove_dir_all(dir);
    }

    if json {
        let out = serde_json::json!({
            "action": "restore",
            "restored": restored,
            "skipped": skipped,
            "failed": failed,
            "skills": results,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
    } else {
        println!(
            "\n{} Restored {}, skipped {}, failed {}",
            if failed == 0 {
                output::green("✓")
            } else {
                output::yellow("!")
            },
            restored,
            skipped,
            failed
        );
    }

    Ok(())
}

struct RestoreEntry {
    name: String,
    source: Option<String>,
    /// Path to skill content in the repo (if available)
    local_path: Option<std::path::PathBuf>,
}

fn read_from_backend() -> Result<Vec<RestoreEntry>, String> {
    let cfg = config::read()?.ok_or_else(|| {
        "No sync backend configured. Run 'equip init' first, or use --from for file restore."
            .to_string()
    })?;

    sync::pull(&cfg)?;

    let ops_dir = config::ops_dir(&cfg)?;
    let skills_dir = config::skills_dir(&cfg)?;
    let state = ops::compute_state(&ops_dir)?;

    Ok(state
        .into_iter()
        .map(|(name, s)| {
            // Check if skill content exists in the repo
            let skill_path = skills_dir.join(&name);
            let local_path = if skill_path.join("SKILL.md").exists() {
                Some(skill_path)
            } else {
                None
            };
            RestoreEntry {
                name,
                source: s.source,
                local_path,
            }
        })
        .collect())
}

/// Pre-clone all unique remote repos in parallel, returning source_str -> temp_dir mapping.
fn pre_clone_repos(skills: &[RestoreEntry], includes: &[String]) -> HashMap<String, PathBuf> {
    // Collect all remote sources and deduplicate by clone URL
    let mut url_to_sources: HashMap<String, Vec<String>> = HashMap::new();
    for source_str in skills
        .iter()
        .filter_map(|s| s.source.as_deref())
        .filter(|s| *s != "adopted")
        .chain(includes.iter().map(|s| s.as_str()))
    {
        if let Ok(source) = SkillSource::parse(source_str)
            && let Some(clone_url) = source.git_clone_url()
        {
            url_to_sources
                .entry(clone_url)
                .or_default()
                .push(source_str.to_string());
        }
    }

    if url_to_sources.is_empty() {
        return HashMap::new();
    }

    // Clone each unique repo in parallel
    let clone_results: Vec<(String, Result<PathBuf, String>)> = std::thread::scope(|s| {
        let handles: Vec<_> = url_to_sources
            .keys()
            .map(|url| {
                let url = url.clone();
                s.spawn(move || {
                    let temp = install::temp_clone_dir();
                    let result = install::clone_repo(&url, &temp);
                    (url, result.map(|()| temp))
                })
            })
            .collect();

        handles
            .into_iter()
            .map(|h| {
                h.join()
                    .unwrap_or_else(|_| ("".into(), Err("thread panicked".into())))
            })
            .collect()
    });

    // Build source_str -> temp_dir mapping
    let mut result: HashMap<String, PathBuf> = HashMap::new();
    for (url, clone_result) in clone_results {
        if let Ok(temp_dir) = clone_result
            && let Some(source_strs) = url_to_sources.get(&url)
        {
            for source_str in source_strs {
                result.insert(source_str.clone(), temp_dir.clone());
            }
        }
    }
    result
}

fn read_from_file(path: &str) -> Result<Vec<RestoreEntry>, String> {
    let content = if path == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin()
            .read_to_string(&mut buf)
            .map_err(|e| format!("Failed to read stdin: {e}"))?;
        buf
    } else {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {path}: {e}"))?
    };

    let entries: Vec<serde_json::Value> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {e}"))?;

    Ok(entries
        .iter()
        .filter_map(|e| {
            let name = e["name"].as_str().unwrap_or("").to_string();
            if name.is_empty() {
                return None; // Skip entries with missing/empty names
            }
            Some(RestoreEntry {
                name,
                source: e["source"].as_str().map(String::from),
                local_path: None,
            })
        })
        .collect())
}
