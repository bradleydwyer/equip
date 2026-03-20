use std::collections::HashMap;
use std::process::Command;

use crate::hash;
use crate::metadata::SkillMetadata;
use crate::output;
use crate::registry;

pub fn run(name: Option<&str>, global: bool, json: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let reg = registry::Registry::load()?;
    let scope = if global {
        registry::scope_global().to_string()
    } else {
        registry::scope_for_project(&project_root)
    };
    let entries = reg.entries_for_scope(&scope);

    let mut to_update: Vec<(String, std::path::PathBuf, SkillMetadata)> = Vec::new();

    for entry in entries {
        if let Some(target) = name
            && entry.skill_name != target
        {
            continue;
        }

        if let Some(path) = registry::find_skill_path(&entry.skill_name, global, &project_root) {
            to_update.push((entry.skill_name.clone(), path, entry.as_metadata()));
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

    // Batch fetch remote HEADs for git sources
    let remote_shas = fetch_remote_shas(&to_update);

    if !json {
        println!("Checking {} skill(s)...\n", to_update.len());
    }

    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut updated = 0;

    for (skill_name, skill_path, meta) in &to_update {
        // Skip adopted skills
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

        // Check if up to date
        if is_up_to_date(meta, &remote_shas) {
            if !json {
                println!(
                    "  {} {} {}",
                    output::green("✓"),
                    output::bold(skill_name),
                    output::dim("up to date")
                );
            }
            results.push(serde_json::json!({
                "name": skill_name,
                "source": meta.source,
                "status": "up_to_date",
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
        match super::install::run(&meta.source, global, &agent_ids, false, false) {
            Ok(()) => {
                crate::telemetry::send("update", Some(skill_name), Some(&meta.source));
                results.push(serde_json::json!({
                    "name": skill_name,
                    "source": meta.source,
                    "status": "updated",
                }));
                updated += 1;
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
    } else if updated == 0 {
        println!("\n{} All skills up to date.", output::green("✓"));
    }

    Ok(())
}

/// Check if a skill is up to date (no upstream changes)
fn is_up_to_date(meta: &SkillMetadata, remote_shas: &HashMap<String, Option<String>>) -> bool {
    // Git sources: compare source_commit to remote HEAD
    if meta.source_type == "git" {
        if let Some(installed_commit) = &meta.source_commit
            && let Some(repo_url) = &meta.repo_url
            && let Some(Some(remote_sha)) = remote_shas.get(repo_url)
        {
            return installed_commit == remote_sha;
        }
        return false;
    }

    // Local sources: compare content_hash to source directory
    if meta.source_type == "local" {
        if let Some(installed_hash) = &meta.content_hash
            && let Some(local_path) = &meta.local_path
        {
            let source_path = std::path::Path::new(local_path);
            if source_path.exists() {
                let source_hash = format!("{:016x}", hash::hash_skill_dir(source_path));
                return &source_hash == installed_hash;
            }
        }
        return false;
    }

    false
}

/// Batch fetch remote HEAD SHAs for git sources
fn fetch_remote_shas(
    skills: &[(String, std::path::PathBuf, SkillMetadata)],
) -> HashMap<String, Option<String>> {
    let mut urls: Vec<String> = skills
        .iter()
        .filter(|(_, _, m)| m.source_type == "git")
        .filter_map(|(_, _, m)| m.repo_url.clone())
        .collect();
    urls.sort();
    urls.dedup();

    let mut results = HashMap::new();
    for url in urls {
        let sha = remote_head_sha(&url);
        results.insert(url, sha);
    }
    results
}

fn remote_head_sha(repo_url: &str) -> Option<String> {
    let output = Command::new("git")
        .args(["ls-remote", "--quiet", repo_url, "HEAD"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.split_whitespace().next().map(String::from)
}
