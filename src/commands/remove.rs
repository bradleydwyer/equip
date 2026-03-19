use crate::agents::{self, AGENTS};
use crate::config;
use crate::ops;
use crate::output;
use crate::sync;

pub fn run(name: &str, global: bool, agent_ids: &[String], json: bool) -> Result<(), String> {
    if name.contains('/')
        || name.contains('\\')
        || name == ".."
        || name == "."
        || name.contains("..")
    {
        return Err(format!("Invalid skill name: '{name}'"));
    }

    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let agents_to_check: Vec<&agents::AgentDef> = if !agent_ids.is_empty() {
        agents::find_agents_by_ids(agent_ids)?
    } else {
        AGENTS.iter().collect()
    };

    let mut removed = Vec::new();

    for agent in &agents_to_check {
        let skill_path = agents::skill_dir(agent, global, &project_root)?.join(name);
        if skill_path.exists() {
            std::fs::remove_dir_all(&skill_path)
                .map_err(|e| format!("Failed to remove {}: {e}", skill_path.display()))?;
            removed.push(agent.name);
        }
    }

    // Update registry
    let mut reg = crate::registry::Registry::load()?;
    let scope = if global {
        crate::registry::scope_global().to_string()
    } else {
        crate::registry::scope_for_project(&project_root)
    };
    if agents_to_check.len() < AGENTS.len() {
        // Partial removal — specific agents
        let removed_ids: Vec<String> = agents_to_check
            .iter()
            .filter(|a| removed.contains(&a.name))
            .map(|a| a.id.to_string())
            .collect();
        reg.remove_agents(&scope, name, &removed_ids);
    } else {
        reg.remove_entry(&scope, name);
    }
    reg.save()?;

    if removed.is_empty() {
        let scope = if global {
            "globally"
        } else {
            "in this project"
        };
        return Err(format!(
            "Skill '{}' not found {}. Run 'equip list' to see installed skills.",
            name, scope
        ));
    }

    if json {
        let out = serde_json::json!({
            "action": "remove",
            "name": name,
            "global": global,
            "removed_from": removed,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
    } else {
        println!(
            "{} Removed '{}' from: {}",
            output::green("✓"),
            output::bold(name),
            removed.join(", ")
        );
    }

    // Auto-sync: write remove op if backend is configured and this is a global remove
    if global && let Ok(Some(cfg)) = config::read() {
        let op = ops::remove_op(name);
        if let Err(e) = sync::write_and_push(&cfg, &op)
            && !json
        {
            eprintln!("{}", output::dim(&format!("Sync: {e}")));
        }
    }

    Ok(())
}
