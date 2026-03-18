use std::path::Path;

use crate::agents::AGENTS;
use crate::output;
use crate::skill;

pub fn run(output_path: Option<&str>, json: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;
    let output_file = project_root.join(output_path.unwrap_or("AGENTS.md"));

    let mut skills: Vec<(String, String)> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    for agent in AGENTS {
        let dir = project_root.join(agent.project_dir);
        if !dir.exists() {
            continue;
        }
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if !path.is_dir() || !path.join("SKILL.md").exists() || seen.contains(&name) {
                continue;
            }
            let description = skill::read_skill(&path)
                .map(|fm| fm.description)
                .unwrap_or_default();
            seen.insert(name.clone());
            skills.push((name, description));
        }
    }

    skills.sort_by(|a, b| a.0.cmp(&b.0));

    let xml = generate_xml(&skills);

    if output_file.exists() {
        update_existing(&output_file, &xml)?;
    } else {
        write_new(&output_file, &xml)?;
    }

    if json {
        let skill_entries: Vec<serde_json::Value> = skills
            .iter()
            .map(|(name, desc)| {
                serde_json::json!({
                    "name": name,
                    "description": desc,
                })
            })
            .collect();
        let out = serde_json::json!({
            "action": "sync",
            "output_file": output_file.display().to_string(),
            "skills": skill_entries,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
    } else {
        println!(
            "{} Synced {} skill(s) to {}",
            output::green("✓"),
            skills.len(),
            output_file.display()
        );
    }
    Ok(())
}

fn generate_xml(skills: &[(String, String)]) -> String {
    let mut xml = String::new();
    xml.push_str("<skills_system priority=\"1\">\n\n");
    xml.push_str("## Available Skills\n\n");
    xml.push_str("<usage>\n");
    xml.push_str("When users ask you to perform tasks, check if any of the available skills below can help complete the task more effectively.\n");
    xml.push_str("</usage>\n\n");
    xml.push_str("<available_skills>\n\n");

    for (name, description) in skills {
        xml.push_str("<skill>\n");
        xml.push_str(&format!("<name>{name}</name>\n"));
        xml.push_str(&format!("<description>{description}</description>\n"));
        xml.push_str("<location>project</location>\n");
        xml.push_str("</skill>\n\n");
    }

    xml.push_str("</available_skills>\n\n");
    xml.push_str("</skills_system>");
    xml
}

fn update_existing(path: &Path, xml: &str) -> Result<(), String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;

    let new_content = if let Some(start) = content.find("<skills_system") {
        if let Some(end) = content.find("</skills_system>") {
            let end = end + "</skills_system>".len();
            format!("{}{}{}", &content[..start], xml, &content[end..])
        } else {
            format!("{}{}{}", &content[..start], xml, &content[start..])
        }
    } else {
        format!("{}\n\n{}\n", content.trim_end(), xml)
    };

    std::fs::write(path, new_content)
        .map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

fn write_new(path: &Path, xml: &str) -> Result<(), String> {
    let content = format!("# AGENTS\n\n{xml}\n");
    std::fs::write(path, content).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}
