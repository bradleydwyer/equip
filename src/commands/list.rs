use std::collections::BTreeMap;

use crate::agents::{self, AGENTS};
use crate::metadata::SkillMetadata;
use crate::output;
use crate::skill;

struct InstalledSkill {
    description: String,
    agents: Vec<&'static str>,
    global: bool,
    managed: bool,
    source: Option<String>,
}

pub fn run(global: bool, json: bool, short: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

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
            let meta = SkillMetadata::read(&path).ok();
            let managed = meta.is_some();
            let source = meta.map(|m| m.source);

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
        print_table(&skills, global, short);
    }

    Ok(())
}

fn print_table(skills: &BTreeMap<String, InstalledSkill>, global: bool, short: bool) {
    let scope = if global { "global" } else { "project" };
    println!("Installed skills ({scope}):\n");

    let unmanaged: Vec<&String> = skills
        .iter()
        .filter(|(_, info)| !info.managed)
        .map(|(name, _)| name)
        .collect();

    for (name, info) in skills {
        let agents_str = format!("[{}]", info.agents.join(", "));
        let tag = if info.managed {
            "".to_string()
        } else {
            format!(" {}", output::yellow("(unmanaged)"))
        };
        println!(
            "  {:<28} {}{}",
            output::bold(name),
            output::dim(&agents_str),
            tag
        );
        if !short && !info.description.is_empty() {
            println!("    {}", output::dim(&info.description));
        }
    }

    println!("\n{} {} skill(s)", output::dim("Total:"), skills.len());

    if !unmanaged.is_empty() {
        println!(
            "\n{} {} skill(s) not managed by equip: {}",
            output::yellow("!"),
            unmanaged.len(),
            unmanaged
                .iter()
                .map(|n| output::bold(n))
                .collect::<Vec<_>>()
                .join(", ")
        );
        println!(
            "  To manage them, reinstall with: {}",
            output::dim("equip install <source>")
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
