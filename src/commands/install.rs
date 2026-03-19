use std::cell::RefCell;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::agents;
use crate::config;
use crate::hash;
use crate::metadata;
use crate::ops;
use crate::output;
use crate::registry;
use crate::skill;
use crate::source::SkillSource;
use crate::sync;

thread_local! {
    /// Track sources currently being installed to prevent infinite include cycles.
    static INSTALLING: RefCell<HashSet<String>> = RefCell::new(HashSet::new());
}

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
    // Cycle detection for recursive includes
    let already_installing = INSTALLING.with(|s| !s.borrow_mut().insert(source_str.to_string()));
    if already_installing {
        return Ok(()); // Skip silently — already being installed up the call stack
    }

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let source = SkillSource::parse(source_str)?;
    let agents = agents::resolve_agents(agent_ids, all, global, &project_root)?;

    let spinner = if !json
        && !quiet
        && matches!(
            source,
            SkillSource::GitHub { .. } | SkillSource::GitUrl { .. }
        ) {
        Some(output::Spinner::start(source_str))
    } else {
        None
    };

    let resolve_result = resolve_source(&source);

    if let Some(s) = spinner {
        s.stop();
    }

    let (skill_dir, temp_dir, source_info) = resolve_result?;

    let result = do_install(
        &skill_dir,
        &source,
        source_str,
        global,
        &agents,
        agent_ids,
        all,
        &project_root,
        json,
        quiet,
        source_info,
        temp_dir.as_deref(),
    );

    if let Some(temp) = &temp_dir {
        let _ = std::fs::remove_dir_all(temp);
    }

    // Remove from cycle detection set
    INSTALLING.with(|s| s.borrow_mut().remove(source_str));

    result
}

#[allow(clippy::too_many_arguments)]
fn do_install(
    skill_dir: &Path,
    source: &SkillSource,
    source_str: &str,
    global: bool,
    agents: &[&agents::AgentDef],
    agent_ids: &[String],
    all: bool,
    project_root: &Path,
    json: bool,
    quiet: bool,
    source_info: SourceInfo,
    temp_dir: Option<&Path>,
) -> Result<(), String> {
    let skills = skill::discover_skills(skill_dir)?;

    // Resolve includes upfront so we can show total count
    let includes_path = skill_dir.join("includes");
    let includes = if includes_path.exists() {
        skill::read_includes(&includes_path).unwrap_or_default()
    } else {
        Vec::new()
    };

    let total = skills.len() + includes.len();
    if !json && !quiet {
        println!(
            "Found {} skill(s), installing to {} agent(s)...\n",
            total,
            agents.len()
        );
    }

    let agent_ids_list: Vec<String> = agents.iter().map(|a| a.id.to_string()).collect();
    let mut installed: Vec<serde_json::Value> = Vec::new();

    for (path, fm) in &skills {
        let skill_name = resolve_skill_name(path, &fm.name, temp_dir);
        validate_skill_name(&skill_name)?;

        // Rename detection: if this source was previously installed under a different name,
        // remove the old directories and registry entry.
        // Skip for multi-skill repos — each skill is a separate identity, not a rename.
        let scope = if global {
            registry::scope_global().to_string()
        } else {
            registry::scope_for_project(project_root)
        };
        if skills.len() == 1 {
            let reg = registry::Registry::load()?;
            if let Some(old_entry) = reg.find_unique_by_source(&scope, source_str)
                && old_entry.skill_name != skill_name
            {
                let old_name = old_entry.skill_name.clone();
                if !json && !quiet {
                    println!(
                        "  {} renamed → {} (removing old directories)",
                        output::dim(&old_name),
                        output::bold(&skill_name),
                    );
                }
                // Remove old skill directories from all agents
                for agent in agents {
                    let old_target =
                        agents::skill_dir(agent, global, project_root)?.join(&old_name);
                    if old_target.exists() {
                        let _ = std::fs::remove_dir_all(&old_target);
                    }
                }
                // Remove old registry entry
                let mut reg = reg;
                reg.remove_entry(&scope, &old_name);
                reg.save()?;
            }
        }

        let mut agent_names = Vec::new();
        let mut agent_paths = Vec::new();
        let mut content_hash = None;
        for agent in agents {
            let target = agents::skill_dir(agent, global, project_root)?.join(&skill_name);
            copy_skill(path, &target)?;

            // Delete any legacy .equip.json sidecar
            let equip_json = target.join(".equip.json");
            if equip_json.exists() {
                let _ = std::fs::remove_file(&equip_json);
            }

            content_hash = Some(format!("{:016x}", hash::hash_skill_dir(&target)));
            agent_names.push(agent.name);
            agent_paths.push(target.display().to_string());
        }

        // Registry upsert
        let mut reg = registry::Registry::load()?;
        let scope = if global {
            registry::scope_global().to_string()
        } else {
            registry::scope_for_project(project_root)
        };
        reg.upsert(registry::RegistryEntry {
            skill_name: skill_name.clone(),
            scope,
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
        });
        reg.save()?;

        installed.push(serde_json::json!({
            "name": skill_name,
            "description": fm.description,
            "agents": agent_names,
            "paths": agent_paths,
        }));

        if !json && !quiet {
            println!("  {} {}", output::bold(&skill_name), output::green("✓"));
        }
    }

    // Install includes as part of the same flow
    for inc_source in &includes {
        let spinner = if !json && !quiet {
            Some(output::Spinner::start(inc_source))
        } else {
            None
        };

        let result = run_quiet(inc_source, global, agent_ids, all);

        if let Some(s) = spinner {
            s.stop();
        }

        match result {
            Ok(()) => {
                if !json && !quiet {
                    println!("  {} {}", output::bold(inc_source), output::green("✓"));
                }
                installed.push(serde_json::json!({
                    "name": inc_source,
                    "status": "installed",
                }));
            }
            Err(e) => {
                if !json && !quiet {
                    eprintln!(
                        "  {} {}",
                        output::bold(inc_source),
                        output::red(&format!("✗ {e}"))
                    );
                }
            }
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

    // Auto-sync: write ops if backend is configured and this is a global install
    if global && let Ok(Some(cfg)) = config::read() {
        for (path, fm) in &skills {
            let skill_name = resolve_skill_name(path, &fm.name, temp_dir);
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
    copy_skill_files(src, dest)
}

/// Copy only skill-relevant files from src to dest.
/// Allowlist: SKILL.md, LICENSE*, and known skill directories.
/// Everything else (source code, CI, build files) is skipped.
fn copy_skill_files(src: &Path, dest: &Path) -> Result<(), String> {
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if src_path.is_dir() {
            if is_skill_dir(&name) {
                let dest_path = dest.join(&file_name);
                std::fs::create_dir_all(&dest_path)
                    .map_err(|e| format!("Failed to create {}: {e}", dest_path.display()))?;
                copy_dir_recursive(&src_path, &dest_path)?;
            }
        } else if is_skill_file(&name) {
            let dest_path = dest.join(&file_name);
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {}: {e}", src_path.display()))?;
        }
    }
    Ok(())
}

/// Directories that are part of a skill (not project infrastructure).
fn is_skill_dir(name: &str) -> bool {
    matches!(
        name,
        "references" | "scripts" | "agents" | "assets" | "evals" | "eval-viewer"
    )
}

/// Files that are part of a skill.
fn is_skill_file(name: &str) -> bool {
    name == "SKILL.md" || name.starts_with("LICENSE") || name.starts_with("license")
}

/// Full recursive copy for skill subdirectories (references/, scripts/, etc.)
fn copy_dir_recursive(src: &Path, dest: &Path) -> Result<(), String> {
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let src_path = entry.path();
        let file_name = entry.file_name();

        let name = file_name.to_string_lossy();
        if name.starts_with('.') {
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

fn resolve_skill_name(
    skill_path: &Path,
    frontmatter_name: &str,
    _temp_dir: Option<&Path>,
) -> String {
    // Always prefer the frontmatter name — it's the canonical skill identity.
    // The directory name is just a filesystem artifact (e.g. a skill named
    // "equip" might live in a dir called "skill/").
    if !frontmatter_name.is_empty() {
        return frontmatter_name.to_string();
    }
    skill_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown".to_string())
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
