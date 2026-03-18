use dialoguer::{MultiSelect, Select, theme::ColorfulTheme};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::agents::{self, AGENTS};
use crate::hash;
use crate::metadata::{self, SkillMetadata};
use crate::output;

#[derive(Debug)]
struct SkillInstance {
    agent_id: &'static str,
    agent_name: &'static str,
    path: PathBuf,
    content_hash: u64,
    has_metadata: bool,
    #[allow(dead_code)]
    source: Option<String>,
}

#[derive(Debug, Clone)]
enum Action {
    Spread {
        skill_name: String,
        source_path: PathBuf,
        target_agents: Vec<&'static str>,
    },
    Align {
        skill_name: String,
        canonical_path: PathBuf,
        canonical_agent: &'static str,
        stale_agents: Vec<&'static str>,
    },
    Adopt {
        skill_name: String,
        agent_name: &'static str,
        path: PathBuf,
    },
    Prune {
        skill_name: String,
        agent_name: &'static str,
        path: PathBuf,
    },
}

impl Action {
    fn skill_name(&self) -> &str {
        match self {
            Action::Spread { skill_name, .. }
            | Action::Align { skill_name, .. }
            | Action::Adopt { skill_name, .. }
            | Action::Prune { skill_name, .. } => skill_name,
        }
    }

    fn group_key(&self) -> String {
        match self {
            Action::Spread { target_agents, .. } => {
                format!("spread:{}", target_agents.join(","))
            }
            Action::Align { .. } => "align".to_string(),
            Action::Adopt { .. } => "adopt".to_string(),
            Action::Prune { .. } => "prune".to_string(),
        }
    }

    fn group_description(&self) -> String {
        match self {
            Action::Spread { target_agents, .. } => {
                format!("Spread to {}", target_agents.join(", "))
            }
            Action::Align {
                canonical_agent, ..
            } => format!("Align to {canonical_agent}'s version"),
            Action::Adopt { .. } => "Adopt into equip (write .equip.json)".to_string(),
            Action::Prune { .. } => "Prune from undetected agents".to_string(),
        }
    }

    fn to_json(&self) -> serde_json::Value {
        match self {
            Action::Spread {
                skill_name,
                source_path,
                target_agents,
            } => serde_json::json!({
                "action": "spread",
                "skill": skill_name,
                "source_path": source_path.display().to_string(),
                "target_agents": target_agents,
            }),
            Action::Align {
                skill_name,
                canonical_path,
                canonical_agent,
                stale_agents,
            } => serde_json::json!({
                "action": "align",
                "skill": skill_name,
                "canonical_path": canonical_path.display().to_string(),
                "canonical_agent": canonical_agent,
                "stale_agents": stale_agents,
            }),
            Action::Adopt {
                skill_name,
                agent_name,
                path,
            } => serde_json::json!({
                "action": "adopt",
                "skill": skill_name,
                "agent": agent_name,
                "path": path.display().to_string(),
            }),
            Action::Prune {
                skill_name,
                agent_name,
                path,
            } => serde_json::json!({
                "action": "prune",
                "skill": skill_name,
                "agent": agent_name,
                "path": path.display().to_string(),
            }),
        }
    }
}

/// A group of actions of the same type that can be batch-applied
struct ActionGroup {
    description: String,
    actions: Vec<Action>,
}

pub fn run(global: bool, json: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let detected = agents::detect_agents(true, &project_root)?;
    let detected_ids: BTreeSet<&str> = detected.iter().map(|a| a.id).collect();

    let mut skills: BTreeMap<String, Vec<SkillInstance>> = BTreeMap::new();

    if global {
        // Global only
        scan_scope(&mut skills, true, &project_root)?;
    } else {
        // Default: both project and global (same as survey)
        scan_scope(&mut skills, false, &project_root)?;
        scan_scope(&mut skills, true, &project_root)?;
    }

    let actions = build_plan(&skills, &detected_ids)?;

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
            println!("{} No issues to fix.", output::green("✓"));
        }
        return Ok(());
    }

    if json {
        print_plan_json(&actions)?;
    } else {
        run_interactive(&actions, global, &project_root)?;
    }

    Ok(())
}

fn build_plan(
    skills: &BTreeMap<String, Vec<SkillInstance>>,
    detected_ids: &BTreeSet<&str>,
) -> Result<Vec<Action>, String> {
    let mut actions = Vec::new();

    for (name, instances) in skills {
        let agent_ids: BTreeSet<&str> = instances.iter().map(|i| i.agent_id).collect();

        // Coverage gaps → Spread
        if agent_ids.len() < detected_ids.len() {
            let missing: Vec<&'static str> = AGENTS
                .iter()
                .filter(|a| detected_ids.contains(a.id) && !agent_ids.contains(a.id))
                .map(|a| a.id)
                .collect();
            if !missing.is_empty()
                && let Some(source_inst) = instances.first()
            {
                actions.push(Action::Spread {
                    skill_name: name.clone(),
                    source_path: source_inst.path.clone(),
                    target_agents: missing,
                });
            }
        }

        // Content mismatches → Align
        let unique_hashes: BTreeSet<u64> = instances.iter().map(|i| i.content_hash).collect();
        if unique_hashes.len() > 1 {
            let canonical = instances
                .iter()
                .find(|i| i.has_metadata)
                .or(instances.first())
                .unwrap();
            let stale: Vec<&'static str> = instances
                .iter()
                .filter(|i| i.content_hash != canonical.content_hash)
                .map(|i| i.agent_name)
                .collect();
            if !stale.is_empty() {
                actions.push(Action::Align {
                    skill_name: name.clone(),
                    canonical_path: canonical.path.clone(),
                    canonical_agent: canonical.agent_name,
                    stale_agents: stale,
                });
            }
        }

        // Unmanaged → Adopt
        for inst in instances.iter().filter(|i| !i.has_metadata) {
            actions.push(Action::Adopt {
                skill_name: name.clone(),
                agent_name: inst.agent_name,
                path: inst.path.clone(),
            });
        }

        // Orphaned → Prune
        for inst in instances
            .iter()
            .filter(|i| !detected_ids.contains(i.agent_id))
        {
            actions.push(Action::Prune {
                skill_name: name.clone(),
                agent_name: inst.agent_name,
                path: inst.path.clone(),
            });
        }
    }

    Ok(actions)
}

fn group_actions(actions: &[Action]) -> Vec<ActionGroup> {
    let mut groups: Vec<ActionGroup> = Vec::new();

    for action in actions {
        let key = action.group_key();
        if let Some(group) = groups.iter_mut().find(|g| {
            g.actions
                .first()
                .map(|a| a.group_key() == key)
                .unwrap_or(false)
        }) {
            group.actions.push(action.clone());
        } else {
            groups.push(ActionGroup {
                description: action.group_description(),
                actions: vec![action.clone()],
            });
        }
    }

    groups
}

fn print_plan_json(actions: &[Action]) -> Result<(), String> {
    let plan: Vec<serde_json::Value> = actions.iter().map(|a| a.to_json()).collect();
    let out = serde_json::json!({
        "action": "fix",
        "plan": plan,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&out).map_err(|e| format!("Failed to serialize JSON: {e}"))?
    );
    Ok(())
}

/// Group actions by unique skill name within a group, returning (skill_name, action_indices)
fn unique_skills(group: &ActionGroup) -> Vec<(String, Vec<usize>)> {
    let mut seen: Vec<(String, Vec<usize>)> = Vec::new();
    for (i, action) in group.actions.iter().enumerate() {
        let name = action.skill_name().to_string();
        if let Some(entry) = seen.iter_mut().find(|(n, _)| *n == name) {
            entry.1.push(i);
        } else {
            seen.push((name, vec![i]));
        }
    }
    seen
}

/// Execute all actions for a skill within a group, return true if all succeeded
fn execute_skill_actions(
    group: &ActionGroup,
    indices: &[usize],
    global: bool,
    project_root: &Path,
) -> Result<(), String> {
    for &idx in indices {
        execute_action(&group.actions[idx], global, project_root)?;
    }
    Ok(())
}

fn run_interactive(actions: &[Action], global: bool, project_root: &Path) -> Result<(), String> {
    let groups = group_actions(actions);
    let theme = ColorfulTheme::default();
    let mut total_applied = 0;
    let mut total_skipped = 0;
    let mut apply_rest = false;

    for (gi, group) in groups.iter().enumerate() {
        let skills = unique_skills(group);
        let count = skills.len();

        println!(
            "\n{} {} ({} skill{})",
            output::bold(&group.description),
            output::dim(&format!("[group {}/{}]", gi + 1, groups.len())),
            count,
            if count == 1 { "" } else { "s" }
        );

        if apply_rest {
            for (name, indices) in &skills {
                match execute_skill_actions(group, indices, global, project_root) {
                    Ok(()) => {
                        println!("  {} {}", output::green("✓"), name);
                        total_applied += 1;
                    }
                    Err(e) => {
                        println!("  {} {} — {e}", output::red("✗"), name);
                        total_skipped += 1;
                    }
                }
            }
            continue;
        }

        if count == 1 {
            let (name, indices) = &skills[0];
            let choices = vec!["Apply", "Skip"];
            let selection = Select::with_theme(&theme)
                .with_prompt(name.as_str())
                .items(&choices)
                .default(0)
                .interact_opt()
                .map_err(|e| format!("Input error: {e}"))?;

            match selection {
                Some(0) => match execute_skill_actions(group, indices, global, project_root) {
                    Ok(()) => {
                        println!("  {}", output::green("✓ Done"));
                        total_applied += 1;
                    }
                    Err(e) => {
                        println!("  {} {e}", output::red("✗"));
                        total_skipped += 1;
                    }
                },
                _ => {
                    println!("  Skipped.");
                    total_skipped += 1;
                }
            }
        } else {
            // Show unique skill names
            for (name, _) in &skills {
                println!("  {} {}", output::dim("·"), name);
            }
            println!();

            let choices = vec![
                format!("Apply to all {count} skills"),
                "Select individually".to_string(),
                "Skip all".to_string(),
            ];

            let selection = Select::with_theme(&theme)
                .with_prompt("How to handle this group?")
                .items(&choices)
                .default(0)
                .interact_opt()
                .map_err(|e| format!("Input error: {e}"))?;

            match selection {
                Some(0) => {
                    for (name, indices) in &skills {
                        match execute_skill_actions(group, indices, global, project_root) {
                            Ok(()) => {
                                println!("  {} {}", output::green("✓"), name);
                                total_applied += 1;
                            }
                            Err(e) => {
                                println!("  {} {} — {e}", output::red("✗"), name);
                                total_skipped += 1;
                            }
                        }
                    }

                    if gi + 1 < groups.len() {
                        let remaining = groups.len() - gi - 1;
                        let rest_choices = vec![
                            format!(
                                "Yes, apply all remaining ({remaining} group{})",
                                if remaining == 1 { "" } else { "s" }
                            ),
                            "No, continue reviewing".to_string(),
                        ];
                        let rest = Select::with_theme(&theme)
                            .with_prompt("Apply same decision to remaining groups?")
                            .items(&rest_choices)
                            .default(1)
                            .interact_opt()
                            .map_err(|e| format!("Input error: {e}"))?;
                        if rest == Some(0) {
                            apply_rest = true;
                        }
                    }
                }
                Some(1) => {
                    // Multi-select by unique skill name
                    let labels: Vec<&str> = skills.iter().map(|(n, _)| n.as_str()).collect();
                    let defaults: Vec<bool> = vec![true; labels.len()];

                    let selected = MultiSelect::with_theme(&theme)
                        .with_prompt("Select skills (Space to toggle, Enter to confirm)")
                        .items(&labels)
                        .defaults(&defaults)
                        .interact_opt()
                        .map_err(|e| format!("Input error: {e}"))?;

                    match selected {
                        Some(selected_indices) => {
                            for (si, (name, action_indices)) in skills.iter().enumerate() {
                                if selected_indices.contains(&si) {
                                    match execute_skill_actions(
                                        group,
                                        action_indices,
                                        global,
                                        project_root,
                                    ) {
                                        Ok(()) => {
                                            println!("  {} {}", output::green("✓"), name);
                                            total_applied += 1;
                                        }
                                        Err(e) => {
                                            println!("  {} {} — {e}", output::red("✗"), name);
                                            total_skipped += 1;
                                        }
                                    }
                                } else {
                                    total_skipped += 1;
                                }
                            }
                        }
                        None => {
                            total_skipped += count;
                            println!("  Skipped all.");
                        }
                    }
                }
                _ => {
                    total_skipped += count;
                    println!("  Skipped all.");
                }
            }
        }
    }

    println!(
        "\n{} Applied {}, skipped {}.",
        if total_applied > 0 {
            output::green("✓")
        } else {
            output::dim("·")
        },
        total_applied,
        total_skipped
    );
    Ok(())
}

fn execute_action(action: &Action, global: bool, project_root: &Path) -> Result<(), String> {
    match action {
        Action::Spread {
            skill_name,
            source_path,
            target_agents,
        } => {
            for agent_id in target_agents {
                let agent = AGENTS
                    .iter()
                    .find(|a| a.id == *agent_id)
                    .ok_or_else(|| format!("Agent not found: {agent_id}"))?;
                let target = agents::skill_dir(agent, global, project_root)?.join(skill_name);
                copy_dir(source_path, &target)?;
            }
            Ok(())
        }
        Action::Align {
            skill_name,
            canonical_path,
            stale_agents,
            ..
        } => {
            for agent_name in stale_agents {
                let agent = AGENTS
                    .iter()
                    .find(|a| a.name == *agent_name)
                    .ok_or_else(|| format!("Agent not found: {agent_name}"))?;
                let target = agents::skill_dir(agent, global, project_root)?.join(skill_name);
                if target.exists() {
                    std::fs::remove_dir_all(&target)
                        .map_err(|e| format!("Failed to remove {}: {e}", target.display()))?;
                }
                copy_dir(canonical_path, &target)?;
            }
            Ok(())
        }
        Action::Adopt { path, .. } => {
            let meta = SkillMetadata {
                source: "adopted".to_string(),
                source_type: "local".to_string(),
                repo_url: None,
                subpath: None,
                local_path: Some(path.display().to_string()),
                installed_at: metadata::now_iso8601(),
                agents: vec![],
                equip_version: env!("CARGO_PKG_VERSION").to_string(),
                source_commit: None,
                content_hash: Some(format!("{:016x}", hash::hash_skill_dir(path))),
                version: None,
                source_tag: None,
                commit_date: None,
                source_date: None,
            };
            meta.write(path)
        }
        Action::Prune { path, .. } => std::fs::remove_dir_all(path)
            .map_err(|e| format!("Failed to remove {}: {e}", path.display())),
    }
}

fn scan_scope(
    skills: &mut BTreeMap<String, Vec<SkillInstance>>,
    global: bool,
    project_root: &Path,
) -> Result<(), String> {
    for agent in AGENTS {
        let dir = agents::skill_dir(agent, global, project_root)?;
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
            let content_hash = hash::hash_skill_md(&path);
            let (has_metadata, source) = match SkillMetadata::read(&path) {
                Ok(meta) => (true, Some(meta.source)),
                Err(_) => (false, None),
            };

            skills.entry(name).or_default().push(SkillInstance {
                agent_id: agent.id,
                agent_name: agent.name,
                path,
                content_hash,
                has_metadata,
                source,
            });
        }
    }
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dest)
        .map_err(|e| format!("Failed to create {}: {e}", dest.display()))?;
    for entry in
        std::fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))?
    {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        let name = entry.file_name().to_string_lossy().to_string();
        if name == ".git" {
            continue;
        }
        if src_path.is_dir() {
            copy_dir(&src_path, &dest_path)?;
        } else {
            std::fs::copy(&src_path, &dest_path)
                .map_err(|e| format!("Failed to copy {}: {e}", src_path.display()))?;
        }
    }
    Ok(())
}
