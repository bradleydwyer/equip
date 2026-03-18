use std::path::{Path, PathBuf};
use std::process::Command;

use crate::agents;
use crate::config;
use crate::hash;
use crate::metadata::{self, SkillMetadata};
use crate::ops;
use crate::output;
use crate::skill;
use crate::source::SkillSource;
use crate::sync;

pub fn run(
    source_str: &str,
    global: bool,
    agent_ids: &[String],
    all: bool,
    json: bool,
) -> Result<(), String> {
    run_inner(source_str, global, agent_ids, all, json, false)
}

/// Run install without any output (used by restore)
pub fn run_quiet(
    source_str: &str,
    global: bool,
    agent_ids: &[String],
    all: bool,
) -> Result<(), String> {
    run_inner(source_str, global, agent_ids, all, false, true)
}

fn run_inner(
    source_str: &str,
    global: bool,
    agent_ids: &[String],
    all: bool,
    json: bool,
    quiet: bool,
) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let source = SkillSource::parse(source_str)?;
    let agents = agents::resolve_agents(agent_ids, all, global, &project_root)?;

    let (skill_dir, temp_dir, source_info) = resolve_source(&source)?;

    let result = do_install(
        &skill_dir,
        &source,
        source_str,
        global,
        &agents,
        &project_root,
        json,
        quiet,
        source_info,
    );

    if let Some(temp) = &temp_dir {
        let _ = std::fs::remove_dir_all(temp);
    }

    result
}

#[allow(clippy::too_many_arguments)]
fn do_install(
    skill_dir: &Path,
    source: &SkillSource,
    source_str: &str,
    global: bool,
    agents: &[&agents::AgentDef],
    project_root: &Path,
    json: bool,
    quiet: bool,
    source_info: SourceInfo,
) -> Result<(), String> {
    let skills = skill::discover_skills(skill_dir)?;

    if !json && !quiet {
        println!(
            "Found {} skill(s), installing to {} agent(s)...\n",
            skills.len(),
            agents.len()
        );
    }

    let agent_ids_list: Vec<String> = agents.iter().map(|a| a.id.to_string()).collect();
    let mut installed: Vec<serde_json::Value> = Vec::new();

    for (path, fm) in &skills {
        let skill_name = resolve_skill_name(path, &fm.name);
        validate_skill_name(&skill_name)?;

        if !json && !quiet {
            print!("  {} ", output::bold(&skill_name));
        }

        let mut agent_names = Vec::new();
        let mut agent_paths = Vec::new();
        for agent in agents {
            let target = agents::skill_dir(agent, global, project_root)?.join(&skill_name);
            copy_skill(path, &target)?;

            let content_hash = Some(format!("{:016x}", hash::hash_skill_dir(&target)));
            let meta = SkillMetadata {
                source: source_str.to_string(),
                source_type: match source {
                    SkillSource::Local { .. } => "local".to_string(),
                    _ => "git".to_string(),
                },
                repo_url: source.repo_url(),
                subpath: source.subpath().map(String::from),
                local_path: match source {
                    SkillSource::Local { path } => Some(path.display().to_string()),
                    _ => None,
                },
                installed_at: metadata::now_iso8601(),
                agents: agent_ids_list.clone(),
                equip_version: env!("CARGO_PKG_VERSION").to_string(),
                source_commit: source_info.commit.clone(),
                content_hash,
                version: fm.version.clone(),
                source_tag: source_info.tag.clone(),
                commit_date: source_info.commit_date.clone(),
                source_date: match source {
                    SkillSource::Local { path } => std::fs::metadata(path.join("SKILL.md"))
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(metadata::system_time_to_date),
                    _ => None,
                },
            };
            meta.write(&target)?;
            agent_names.push(agent.name);
            agent_paths.push(target.display().to_string());
        }

        installed.push(serde_json::json!({
            "name": skill_name,
            "description": fm.description,
            "agents": agent_names,
            "paths": agent_paths,
        }));

        if !json && !quiet {
            println!("{}", output::dim(&format!("[{}]", agent_names.join(", "))));
        }
    }

    if !quiet {
        if json {
            let out = serde_json::json!({
                "action": "install",
                "source": source_str,
                "global": global,
                "skills": installed,
            });
            println!(
                "{}",
                serde_json::to_string_pretty(&out)
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            );
        } else {
            println!(
                "\n{} Installed {} skill(s) to {} agent(s)",
                output::green("✓"),
                installed.len(),
                agents.len()
            );
        }
    }

    // Auto-sync: write ops and copy skill content if backend is configured and this is a global install
    if global
        && let Ok(Some(cfg)) = config::read()
        && let Ok(skills_dir) = config::skills_dir(&cfg)
    {
        for (path, fm) in &skills {
            let skill_name = resolve_skill_name(path, &fm.name);
            // Copy skill content to repo
            let dest = skills_dir.join(&skill_name);
            let _ = copy_skill(path, &dest);
            // Write op
            let op = ops::add_op(&skill_name, Some(source_str), &fm.description);
            if let Err(e) = sync::write_and_push(&cfg, &op)
                && !json
            {
                eprintln!("{}", output::dim(&format!("Sync: {e}")));
            }
        }
    }

    Ok(())
}

struct SourceInfo {
    commit: Option<String>,
    tag: Option<String>,
    commit_date: Option<String>,
}

/// Returns (skill_dir, optional_temp_dir_to_cleanup, source_info)
fn resolve_source(source: &SkillSource) -> Result<(PathBuf, Option<PathBuf>, SourceInfo), String> {
    match source {
        SkillSource::Local { path } => {
            if !path.exists() {
                return Err(format!("Path does not exist: {}", path.display()));
            }
            let info = SourceInfo {
                commit: None,
                tag: None,
                commit_date: None,
            };
            Ok((path.clone(), None, info))
        }
        SkillSource::GitHub { .. } | SkillSource::GitUrl { .. } => {
            let clone_url = source.git_clone_url().unwrap();
            let temp = temp_clone_dir();
            clone_repo(&clone_url, &temp)?;

            let info = SourceInfo {
                commit: get_head_sha(&temp),
                tag: get_head_tag(&temp),
                commit_date: get_commit_date(&temp),
            };
            let subpath = source.subpath();

            if let Some(sp) = subpath {
                validate_subpath(sp)?;
                let full = temp.join(sp);
                if !full.exists() {
                    return Err(format!("Subpath '{}' not found in repository", sp));
                }
                Ok((full, Some(temp), info))
            } else {
                Ok((temp.clone(), Some(temp), info))
            }
        }
    }
}

fn get_head_sha(repo_dir: &Path) -> Option<String> {
    Command::new("git")
        .args(["-C", &repo_dir.display().to_string(), "rev-parse", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
}

fn get_head_tag(repo_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_dir.display().to_string(),
            "describe",
            "--tags",
            "--exact-match",
            "HEAD",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let tag = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if tag.is_empty() { None } else { Some(tag) }
}

fn get_commit_date(repo_dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_dir.display().to_string(),
            "log",
            "-1",
            "--format=%aI",
        ])
        .output()
        .ok()
        .filter(|o| o.status.success())?;
    let date_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    metadata::iso8601_to_date(&date_str)
}

fn clone_repo(url: &str, dest: &Path) -> Result<(), String> {
    Command::new("git").arg("--version").output().map_err(|_| {
        "git is not installed or not in PATH. Install git to use GitHub/URL sources.".to_string()
    })?;

    let output = Command::new("git")
        .args(["clone", "--depth", "1", "--quiet", url])
        .arg(dest)
        .output()
        .map_err(|e| format!("Failed to run git clone: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("git clone failed: {}", stderr.trim()));
    }
    Ok(())
}

fn temp_clone_dir() -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    std::env::temp_dir().join(format!("equip-{}", timestamp))
}

fn copy_skill(src: &Path, dest: &Path) -> Result<(), String> {
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

fn resolve_skill_name(skill_path: &Path, frontmatter_name: &str) -> String {
    skill_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| frontmatter_name.to_string())
}

fn validate_skill_name(name: &str) -> Result<(), String> {
    if name.contains('/')
        || name.contains('\\')
        || name == ".."
        || name == "."
        || name.contains("..")
    {
        return Err(format!("Invalid skill name: '{name}'"));
    }
    Ok(())
}

fn validate_subpath(subpath: &str) -> Result<(), String> {
    if subpath.contains("..") {
        return Err(format!(
            "Invalid subpath '{}': path traversal not allowed",
            subpath
        ));
    }
    Ok(())
}
