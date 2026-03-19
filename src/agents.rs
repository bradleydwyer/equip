use std::path::Path;

pub struct AgentDef {
    pub id: &'static str,
    pub name: &'static str,
    /// Relative to project root
    pub project_dir: &'static str,
    /// Relative to $HOME
    pub global_dir: &'static str,
    /// Directory to check for agent presence (relative to $HOME for global, project root for local)
    pub detect_dir: &'static str,
}

pub const AGENTS: &[AgentDef] = &[
    // Prioritized
    AgentDef {
        id: "claude",
        name: "Claude Code",
        project_dir: ".claude/skills",
        global_dir: ".claude/skills",
        detect_dir: ".claude",
    },
    AgentDef {
        id: "codex",
        name: "Codex",
        project_dir: ".codex/skills",
        global_dir: ".codex/skills",
        detect_dir: ".codex",
    },
    AgentDef {
        id: "gemini",
        name: "Gemini CLI",
        project_dir: ".gemini/skills",
        global_dir: ".gemini/skills",
        detect_dir: ".gemini",
    },
    AgentDef {
        id: "opencode",
        name: "OpenCode",
        project_dir: ".opencode/skill",
        global_dir: ".config/opencode/skill",
        detect_dir: ".config/opencode",
    },
    AgentDef {
        id: "pi",
        name: "pi-mono",
        project_dir: ".agents/skills",
        global_dir: ".pi/agent/skills",
        detect_dir: ".pi",
    },
    // Alphabetical
    AgentDef {
        id: "amp",
        name: "Amp",
        project_dir: ".agents/skills",
        global_dir: ".config/agents/skills",
        detect_dir: ".config/agents",
    },
    AgentDef {
        id: "cline",
        name: "Cline",
        project_dir: ".cline/skills",
        global_dir: ".cline/skills",
        detect_dir: ".cline",
    },
    AgentDef {
        id: "continue",
        name: "Continue",
        project_dir: ".continue/skills",
        global_dir: ".continue/skills",
        detect_dir: ".continue",
    },
    AgentDef {
        id: "cursor",
        name: "Cursor",
        project_dir: ".cursor/skills",
        global_dir: ".cursor/skills",
        detect_dir: ".cursor",
    },
    AgentDef {
        id: "copilot",
        name: "GitHub Copilot",
        project_dir: ".github/skills",
        global_dir: ".github/skills",
        detect_dir: ".github",
    },
    AgentDef {
        id: "goose",
        name: "Goose",
        project_dir: ".goose/skills",
        global_dir: ".config/goose/skills",
        detect_dir: ".config/goose",
    },
    AgentDef {
        id: "kilo",
        name: "Kilo Code",
        project_dir: ".kilocode/skills",
        global_dir: ".kilocode/skills",
        detect_dir: ".kilocode",
    },
    AgentDef {
        id: "kiro",
        name: "Kiro",
        project_dir: ".kiro/skills",
        global_dir: ".kiro/skills",
        detect_dir: ".kiro",
    },
    AgentDef {
        id: "pearai",
        name: "Pear AI",
        project_dir: ".pearai/skills",
        global_dir: ".pearai/skills",
        detect_dir: ".pearai",
    },
    AgentDef {
        id: "roo",
        name: "Roo Code",
        project_dir: ".roo/skills",
        global_dir: ".roo/skills",
        detect_dir: ".roo",
    },
    AgentDef {
        id: "cody",
        name: "Sourcegraph Cody",
        project_dir: ".sourcegraph/skills",
        global_dir: ".sourcegraph/skills",
        detect_dir: ".sourcegraph",
    },
    AgentDef {
        id: "windsurf",
        name: "Windsurf",
        project_dir: ".windsurf/skills",
        global_dir: ".codeium/windsurf/skills",
        detect_dir: ".codeium",
    },
    AgentDef {
        id: "zed",
        name: "Zed",
        project_dir: ".zed/skills",
        global_dir: ".zed/skills",
        detect_dir: ".zed",
    },
];

pub fn home_dir() -> Result<std::path::PathBuf, String> {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(std::path::PathBuf::from)
        .map_err(|_| "Could not determine home directory (HOME or USERPROFILE not set)".to_string())
}

pub fn detect_agents(global: bool, project_root: &Path) -> Result<Vec<&'static AgentDef>, String> {
    let home = home_dir()?;
    Ok(AGENTS
        .iter()
        .filter(|a| {
            if global {
                home.join(a.detect_dir).exists()
            } else {
                project_root.join(a.detect_dir).exists()
            }
        })
        .collect())
}

pub fn find_agents_by_ids(ids: &[String]) -> Result<Vec<&'static AgentDef>, String> {
    let mut agents = Vec::new();
    for id in ids {
        let agent = AGENTS.iter().find(|a| a.id == id.as_str()).ok_or_else(|| {
            let valid: Vec<&str> = AGENTS.iter().map(|a| a.id).collect();
            format!("Unknown agent '{}'. Valid agents: {}", id, valid.join(", "))
        })?;
        agents.push(agent);
    }
    Ok(agents)
}

pub fn resolve_agents(
    agent_ids: &[String],
    all: bool,
    global: bool,
    project_root: &Path,
) -> Result<Vec<&'static AgentDef>, String> {
    if !agent_ids.is_empty() {
        return find_agents_by_ids(agent_ids);
    }
    if all {
        return Ok(AGENTS.iter().collect());
    }
    let detected = detect_agents(global, project_root)?;
    if detected.is_empty() {
        return Err(
            "No AI coding agents detected. Use --agent <id> to target a specific agent, or --all to install for all agents."
                .to_string(),
        );
    }
    Ok(detected)
}

pub fn skill_dir(
    agent: &AgentDef,
    global: bool,
    project_root: &Path,
) -> Result<std::path::PathBuf, String> {
    if global {
        Ok(home_dir()?.join(agent.global_dir))
    } else {
        Ok(project_root.join(agent.project_dir))
    }
}
