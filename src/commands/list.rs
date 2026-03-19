use std::collections::BTreeMap;

use crate::agents::{self, AGENTS};
use crate::output;
use crate::registry;
use crate::skill;

struct InstalledSkill {
    description: String,
    agents: Vec<&'static str>,
    global: bool,
    managed: bool,
    source: Option<String>,
}

pub fn run(global: bool, json: bool, long: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let reg = registry::Registry::load()?;
    let scope = if global {
        registry::scope_global().to_string()
    } else {
        registry::scope_for_project(&project_root)
    };

    let mut skills: BTreeMap<String, InstalledSkill> = BTreeMap::new();

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
            if !path.is_dir() || !path.join("SKILL.md").exists() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().to_string();
            let description = skill::read_skill(&path)
                .map(|fm| fm.description)
                .unwrap_or_default();

            // Delete any legacy .equip.json sidecar
            let equip_json = path.join(".equip.json");
            if equip_json.exists() {
                let _ = std::fs::remove_file(&equip_json);
            }

            let reg_entry = reg.get(&scope, &name);
            let managed = reg_entry.is_some();
            let source = reg_entry.map(|e| e.source.clone());

            skills
                .entry(name)
                .and_modify(|s| {
                    if !s.agents.contains(&agent.name) {
                        s.agents.push(agent.name);
                    }
                })
                .or_insert(InstalledSkill {
                    description,
                    agents: vec![agent.name],
                    global,
                    managed,
                    source,
                });
        }
    }

    if skills.is_empty() {
        let scope = if global {
            "globally"
        } else {
            "in this project"
        };
        println!("No skills installed {scope}.");
        return Ok(());
    }

    if json {
        print_json(&skills)?;
    } else {
        print_table(&skills, global, long);
    }

    Ok(())
}

fn print_table(skills: &BTreeMap<String, InstalledSkill>, global: bool, long: bool) {
    let scope = if global { "global" } else { "project" };
    println!("Installed skills ({scope}):\n");

    let total_agents = AGENTS.len();
    let mut unmanaged_count = 0;

    for (name, info) in skills {
        let prefix = if info.managed {
            output::green("✓")
        } else {
            unmanaged_count += 1;
            output::yellow("?")
        };

        let agents_str = if info.agents.len() == total_agents {
            format!("all {} agents", total_agents)
        } else {
            let ids: Vec<&str> = info
                .agents
                .iter()
                .filter_map(|name| AGENTS.iter().find(|a| a.name == *name).map(|a| a.id))
                .collect();
            format!("{} ({}/{})", ids.join(", "), info.agents.len(), total_agents)
        };

        println!(
            "  {} {:<20} {}",
            prefix,
            output::bold(name),
            output::dim(&agents_str),
        );
        if long && !info.description.is_empty() {
            println!("    {}", output::dim(&info.description));
        }
    }

    println!(
        "\n  {} managed   {} unmanaged",
        output::green("✓"),
        output::yellow("?"),
    );
    println!("\n{} {} skill(s)", output::dim("Total:"), skills.len());

    if unmanaged_count > 0 {
        println!(
            "\n{} {} unmanaged skill(s). Reinstall with {} to track them.",
            output::yellow("!"),
            unmanaged_count,
            output::bold("equip install <source>"),
        );
    }
}

fn print_json(skills: &BTreeMap<String, InstalledSkill>) -> Result<(), String> {
    let entries: Vec<serde_json::Value> = skills
        .iter()
        .map(|(name, info)| {
            serde_json::json!({
                "name": name,
                "description": info.description,
                "agents": info.agents,
                "global": info.global,
                "managed": info.managed,
                "source": info.source,
            })
        })
        .collect();
    let json = serde_json::to_string_pretty(&entries)
        .map_err(|e| format!("Failed to serialize JSON: {e}"))?;
    println!("{json}");
    Ok(())
}
