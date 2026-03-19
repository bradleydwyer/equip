use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::agents::{self, AGENTS};
use crate::config;
use crate::hash;
use crate::output;
use crate::registry;
use crate::skill;

struct SkillInstance {
    agent_id: &'static str,
    agent_name: &'static str,
    path: PathBuf,
    scope: String, // "global", "project", or the scanned directory path
    content_hash: u64,
    has_metadata: bool,
    source: Option<String>,
}

pub fn run(global: bool, json: bool, scan_path: Option<&str>, fix: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let detected = agents::detect_agents(true, &project_root)?;
    let detected_ids: BTreeSet<&str> = detected.iter().map(|a| a.id).collect();

    let reg = registry::Registry::load()?;
    let mut skills: BTreeMap<String, Vec<SkillInstance>> = BTreeMap::new();

    // Resolve scan path: explicit --path, or projects_path from settings
    let effective_scan_path = scan_path.map(String::from).or_else(|| {
        if !global {
            config::read_settings().ok().and_then(|s| s.projects_path)
        } else {
            None
        }
    });

    if let Some(ref path) = effective_scan_path {
        let scan_root = PathBuf::from(path)
            .canonicalize()
            .map_err(|e| format!("Invalid path '{}': {e}", path))?;
        scan_directory_tree(&scan_root, &mut skills, &reg)?;
    } else if global {
        // Global only
        scan_scope(&mut skills, true, &project_root, "global", &reg)?;
    } else {
        // Default: both project and global
        let project_scope = registry::scope_for_project(&project_root);
        scan_scope_with_registry_key(&mut skills, false, &project_root, "project", &reg, &project_scope)?;
        scan_scope(&mut skills, true, &project_root, "global", &reg)?;
    }

    if skills.is_empty() {
        let scope_label = if let Some(ref p) = effective_scan_path {
            format!("in {p}")
        } else if global {
            "globally".to_string()
        } else {
            "in this project or globally".to_string()
        };
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "survey",
                    "skills": [],
                    "issues": [],
                }))
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            );
        } else {
            println!("No skills found {scope_label}.");
        }
        return Ok(());
    }

    let mut issues: Vec<Issue> = Vec::new();

    for (name, instances) in &skills {
        let agent_ids: BTreeSet<&str> = instances.iter().map(|i| i.agent_id).collect();

        // Coverage gaps: only check when not scanning a path (path mode finds skills
        // wherever they are, coverage gaps don't apply)
        if effective_scan_path.is_none() && agent_ids.len() < detected_ids.len() {
            let missing: Vec<&str> = detected_ids.difference(&agent_ids).copied().collect();
            if !missing.is_empty() {
                issues.push(Issue {
                    skill: name.clone(),
                    kind: IssueKind::CoverageGap,
                    detail: format!(
                        "installed in {} agent(s) but missing from: {}",
                        agent_ids.len(),
                        missing.join(", ")
                    ),
                });
            }
        }

        // Content mismatches
        let unique_hashes: BTreeSet<u64> = instances.iter().map(|i| i.content_hash).collect();
        if unique_hashes.len() > 1 {
            let groups = content_mismatch_detail(instances);
            issues.push(Issue {
                skill: name.clone(),
                kind: IssueKind::ContentMismatch,
                detail: format!("{} different versions: {}", unique_hashes.len(), groups),
            });
        }

        // Source mismatches
        let unique_sources: BTreeSet<&str> = instances
            .iter()
            .filter_map(|i| i.source.as_deref())
            .collect();
        if unique_sources.len() > 1 {
            let sources_str: Vec<String> =
                unique_sources.iter().map(|s| format!("'{s}'")).collect();
            issues.push(Issue {
                skill: name.clone(),
                kind: IssueKind::SourceMismatch,
                detail: format!(
                    "installed from different sources: {}",
                    sources_str.join(", ")
                ),
            });
        }

        // Unmanaged
        let unmanaged: Vec<&str> = instances
            .iter()
            .filter(|i| !i.has_metadata)
            .map(|i| i.agent_name)
            .collect();
        if !unmanaged.is_empty() {
            issues.push(Issue {
                skill: name.clone(),
                kind: IssueKind::Unmanaged,
                detail: format!(
                    "not tracked in registry: {} (not managed by equip)",
                    unmanaged.join(", ")
                ),
            });
        }

        // Orphaned (only when not in path-scan mode)
        if effective_scan_path.is_none() {
            let orphaned: Vec<&str> = instances
                .iter()
                .filter(|i| !detected_ids.contains(i.agent_id))
                .map(|i| i.agent_name)
                .collect();
            if !orphaned.is_empty() {
                issues.push(Issue {
                    skill: name.clone(),
                    kind: IssueKind::Orphaned,
                    detail: format!(
                        "exists in undetected agent(s): {} (agent may have been uninstalled)",
                        orphaned.join(", ")
                    ),
                });
            }
        }
    }

    if fix {
        // Show survey results first, then enter fix mode
        if !json {
            print_human(
                &skills,
                &issues,
                &detected_ids,
                effective_scan_path.as_deref(),
            );
        }

        // Convert survey scan to fix scan and run fix flow
        let mut fix_skills: BTreeMap<String, Vec<super::fix::SkillInstance>> = BTreeMap::new();
        for (name, instances) in &skills {
            let fix_instances: Vec<super::fix::SkillInstance> = instances
                .iter()
                .map(|i| super::fix::SkillInstance {
                    agent_id: i.agent_id,
                    agent_name: i.agent_name,
                    path: i.path.clone(),
                    content_hash: i.content_hash,
                    has_metadata: i.has_metadata,
                    source: i.source.clone(),
                })
                .collect();
            fix_skills.insert(name.clone(), fix_instances);
        }

        let actions = super::fix::build_plan(&fix_skills, &detected_ids)?;
        if actions.is_empty() {
            if json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "action": "fix",
                        "plan": [],
                        "message": "No issues to fix."
                    }))
                    .map_err(|e| format!("Failed to serialize JSON: {e}"))?
                );
            } else {
                println!("\n{} No issues to fix.", output::green("✓"));
            }
            return Ok(());
        }

        if json {
            super::fix::print_plan_json(&actions)?;
        } else {
            let project_root = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {e}"))?;
            super::fix::run_interactive(&actions, global, &project_root)?;
        }
    } else if json {
        print_json(&skills, &issues)?;
    } else {
        print_human(
            &skills,
            &issues,
            &detected_ids,
            effective_scan_path.as_deref(),
        );
    }

    Ok(())
}

fn scan_scope(
    skills: &mut BTreeMap<String, Vec<SkillInstance>>,
    global: bool,
    project_root: &Path,
    scope_label: &str,
    reg: &registry::Registry,
) -> Result<(), String> {
    let registry_scope = if global {
        registry::scope_global().to_string()
    } else {
        registry::scope_for_project(project_root)
    };
    for agent in AGENTS {
        let dir = agents::skill_dir(agent, global, project_root)?;
        collect_skills_from_dir(&dir, agent, scope_label, skills, reg, &registry_scope);
    }
    Ok(())
}

fn scan_scope_with_registry_key(
    skills: &mut BTreeMap<String, Vec<SkillInstance>>,
    global: bool,
    project_root: &Path,
    scope_label: &str,
    reg: &registry::Registry,
    registry_scope: &str,
) -> Result<(), String> {
    for agent in AGENTS {
        let dir = agents::skill_dir(agent, global, project_root)?;
        collect_skills_from_dir(&dir, agent, scope_label, skills, reg, registry_scope);
    }
    Ok(())
}

fn scan_directory_tree(
    root: &Path,
    skills: &mut BTreeMap<String, Vec<SkillInstance>>,
    reg: &registry::Registry,
) -> Result<(), String> {
    // Walk subdirectories of root, looking for project-level agent skill dirs
    // e.g., ~/dev/project-a/.claude/skills/, ~/dev/project-b/.cursor/skills/
    let entries =
        std::fs::read_dir(root).map_err(|e| format!("Failed to read {}: {e}", root.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let registry_scope = path
            .canonicalize()
            .unwrap_or_else(|_| path.clone())
            .display()
            .to_string();

        // Check if this subdirectory is itself a project with agent dirs
        for agent in AGENTS {
            let skill_dir = path.join(agent.project_dir);
            if skill_dir.exists() {
                let scope = path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| path.display().to_string());
                collect_skills_from_dir(&skill_dir, agent, &scope, skills, reg, &registry_scope);
            }
        }
    }

    // Also check root itself as a project
    let root_registry_scope = root
        .canonicalize()
        .unwrap_or_else(|_| root.to_path_buf())
        .display()
        .to_string();
    for agent in AGENTS {
        let skill_dir = root.join(agent.project_dir);
        if skill_dir.exists() {
            let scope = root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| root.display().to_string());
            collect_skills_from_dir(&skill_dir, agent, &scope, skills, reg, &root_registry_scope);
        }
    }

    Ok(())
}

fn collect_skills_from_dir(
    dir: &Path,
    agent: &'static agents::AgentDef,
    scope_label: &str,
    skills: &mut BTreeMap<String, Vec<SkillInstance>>,
    reg: &registry::Registry,
    registry_scope: &str,
) {
    if !dir.exists() {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() || !path.join("SKILL.md").exists() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let content_hash = hash::hash_skill_md(&path);

        // Delete any legacy .equip.json sidecar
        let equip_json = path.join(".equip.json");
        if equip_json.exists() {
            let _ = std::fs::remove_file(&equip_json);
        }

        let reg_entry = reg.get(registry_scope, &name);
        let has_metadata = reg_entry.is_some();
        let source = reg_entry.map(|e| e.source.clone());

        skills.entry(name).or_default().push(SkillInstance {
            agent_id: agent.id,
            agent_name: agent.name,
            path,
            scope: scope_label.to_string(),
            content_hash,
            has_metadata,
            source,
        });
    }
}

fn print_human(
    skills: &BTreeMap<String, Vec<SkillInstance>>,
    issues: &[Issue],
    detected_ids: &BTreeSet<&str>,
    scan_path: Option<&str>,
) {
    let scope = if let Some(p) = scan_path {
        format!("path: {p}")
    } else {
        format!("{} detected agent(s)", detected_ids.len())
    };
    println!("Survey: {} skill(s) across {}\n", skills.len(), scope);

    for (name, instances) in skills {
        let locations: Vec<String> = instances
            .iter()
            .map(|i| {
                if scan_path.is_some() {
                    format!("{}/{}", i.scope, i.agent_name)
                } else {
                    format!("{} ({})", i.agent_name, i.scope)
                }
            })
            .collect();

        let desc = instances
            .first()
            .and_then(|i| skill::read_skill(&i.path).ok())
            .map(|fm| fm.description)
            .unwrap_or_default();

        println!(
            "  {:<24} {}",
            output::bold(name),
            output::dim(&locations.join(", "))
        );
        if !desc.is_empty() {
            println!("    {}", output::dim(&desc));
        }
    }

    if issues.is_empty() {
        println!("\n{} No issues found.", output::green("✓"));
    } else {
        println!(
            "\n{} {} issue(s) found:\n",
            output::yellow("!"),
            issues.len()
        );
        for issue in issues {
            let label = match issue.kind {
                IssueKind::CoverageGap => "coverage gap",
                IssueKind::ContentMismatch => "content mismatch",
                IssueKind::SourceMismatch => "source mismatch",
                IssueKind::Unmanaged => "unmanaged",
                IssueKind::Orphaned => "orphaned",
            };
            println!(
                "  {} [{}] {}",
                output::bold(&issue.skill),
                output::yellow(label),
                issue.detail
            );
        }
    }
}

fn print_json(
    skills: &BTreeMap<String, Vec<SkillInstance>>,
    issues: &[Issue],
) -> Result<(), String> {
    let skill_entries: Vec<serde_json::Value> = skills
        .iter()
        .map(|(name, instances)| {
            let agents: Vec<serde_json::Value> = instances
                .iter()
                .map(|i| {
                    serde_json::json!({
                        "agent_id": i.agent_id,
                        "agent_name": i.agent_name,
                        "path": i.path.display().to_string(),
                        "scope": i.scope,
                        "content_hash": format!("{:016x}", i.content_hash),
                        "has_metadata": i.has_metadata,
                        "source": i.source,
                    })
                })
                .collect();
            serde_json::json!({
                "name": name,
                "instances": agents,
            })
        })
        .collect();

    let issue_entries: Vec<serde_json::Value> = issues
        .iter()
        .map(|i| {
            serde_json::json!({
                "skill": i.skill,
                "kind": match i.kind {
                    IssueKind::CoverageGap => "coverage_gap",
                    IssueKind::ContentMismatch => "content_mismatch",
                    IssueKind::SourceMismatch => "source_mismatch",
                    IssueKind::Unmanaged => "unmanaged",
                    IssueKind::Orphaned => "orphaned",
                },
                "detail": i.detail,
            })
        })
        .collect();

    let out = serde_json::json!({
        "action": "survey",
        "skills": skill_entries,
        "issues": issue_entries,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| format!("Failed to serialize JSON: {e}"))?
    );
    Ok(())
}

fn content_mismatch_detail(instances: &[SkillInstance]) -> String {
    let mut groups: BTreeMap<u64, Vec<String>> = BTreeMap::new();
    for inst in instances {
        let label = if inst.scope == "global" || inst.scope == "project" {
            format!("{} ({})", inst.agent_name, inst.scope)
        } else {
            format!("{}/{}", inst.scope, inst.agent_name)
        };
        groups.entry(inst.content_hash).or_default().push(label);
    }
    groups
        .values()
        .map(|agents| format!("[{}]", agents.join(", ")))
        .collect::<Vec<_>>()
        .join(" vs ")
}

struct Issue {
    skill: String,
    kind: IssueKind,
    detail: String,
}

#[derive(Clone, Copy)]
enum IssueKind {
    CoverageGap,
    ContentMismatch,
    SourceMismatch,
    Unmanaged,
    Orphaned,
}
