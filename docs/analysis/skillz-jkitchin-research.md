# Skillz (jkitchin/skillz) -- Comprehensive Research

**Date:** 2026-03-17
**Repo:** https://github.com/jkitchin/skillz
**Author:** John Kitchin (jkitchin@andrew.cmu.edu, Carnegie Mellon University)
**Language:** Python
**License:** MIT
**Stars:** 22
**Forks:** 3
**Created:** 2025-11-29
**Last pushed:** 2026-03-07
**Version:** 0.1.0 (alpha)
**Status:** Active development, alpha-stage

---

## Summary

Skillz is a Python CLI tool for managing AI assistant "skills" (system prompt injections), slash commands, lifecycle hooks, and subagent definitions across multiple LLM coding platforms. It provides a local repository model where skills are directories containing markdown files with YAML frontmatter, and the CLI copies them to the correct platform-specific directories.

It also includes a CASCADE-inspired skill evolution system that can harvest skills from session transcripts, optimize skill descriptions, A/B test variants, track usage statistics, and merge overlapping skills.

---

## Supported Agents/IDEs (Full List)

| Platform | Skills Dir | Commands Dir | Status |
|----------|-----------|-------------|--------|
| **Claude Code** | `~/.claude/skills/` | `~/.claude/commands/` | Tested, primary target |
| **OpenCode** | `~/.config/opencode/skills/` | `~/.config/opencode/command/` | Tested |
| **Codex CLI** (OpenAI) | `~/.codex/skills/` | `~/.codex/commands/` | Configured, untested ("please report issues") |
| **Gemini** (Google) | `~/.config/gemini/skills/` | `~/.config/gemini/commands/` | Configured, untested |

Note: The author recommends https://www.npmjs.com/package/opencode-skills for OpenCode users.

---

## How Skills Are Defined

### Skills (directories with SKILL.md)
```
my-skill/
  SKILL.md          # Required: YAML frontmatter + markdown instructions
  README.md         # Optional
  QUICK_REFERENCE.md # Optional
  examples/         # Optional
  references/       # Optional
  scripts/          # Optional
```

**SKILL.md format:**
```markdown
---
name: my-skill                    # Required: lowercase, hyphens, max 64 chars
description: What this skill does # Required: max 1024 chars
allowed-tools: ["*"]             # Optional: restrict available tools
---

# Skill Content
Detailed instructions for the LLM...
```

### Commands (standalone .md files)
```markdown
---
description: Brief description      # Optional: max 256 chars, shown in /help
model: sonnet                       # Optional: sonnet, opus, or haiku
allowed-tools: ["*"]               # Optional
argument-hint: <your-arg>          # Optional: autocomplete hint
disable-model-invocation: false    # Optional
---
Command prompt content here...
Use $ARGUMENTS or $1, $2, etc. for parameters.
```

### Hooks (directories with HOOK.md + script)
```
my-hook/
  HOOK.md     # Required: YAML frontmatter (name, description, event, matcher, type, timeout)
  hook.py     # Required: executable script (Python or shell)
```

**HOOK.md frontmatter fields:**
- `name` (required): lowercase, hyphens, max 64 chars
- `description` (required): max 256 chars
- `event` (required): PreToolUse, PostToolUse, PermissionRequest, UserPromptSubmit, Notification, Stop, SubagentStop, PreCompact, SessionStart, SessionEnd
- `matcher` (optional): regex pattern for tool names (e.g., `Edit|Write`)
- `type` (optional): `command` or `prompt`
- `timeout` (optional): max execution time in seconds

### Agents (standalone .md files)
```markdown
---
name: my-agent
description: What this agent does
tools: Read, Write, Edit, Grep, Glob
model: sonnet
disallowedTools: []               # Optional
---
Agent instructions...
```

---

## Installation Method

**Not published as a pip-installable package on PyPI under jkitchin's name.**

The PyPI package `skillz` (versions 0.1.0-0.1.14) is a *different project* by Eleanor Berger (intellectronica) -- an MCP server for exposing skills to MCP clients. Not the same tool.

**jkitchin/skillz installation requires cloning the repo:**

```bash
git clone https://github.com/jkitchin/skillz.git
cd skillz

# Using uv (recommended)
uv pip install -e .

# Using pip
pip install -e .
```

**Dependencies:** click>=8.0, pyyaml>=6.0, rich>=13.0
**Optional deps:** gitpython (git features), jinja2 (templates)
**Python:** >=3.10
**Build system:** hatchling

---

## CLI Commands Available

### Core Commands
| Command | Description |
|---------|-------------|
| `skillz install <name>` | Install a skill or command (supports `--all`, `--platform`, `--target`, `--force`, `--dry-run`) |
| `skillz uninstall <name>` | Uninstall a skill or command |
| `skillz list` | List skills and commands (filter by `--type`, `--source`) |
| `skillz search <query>` | Search skills/commands by keyword in names and descriptions |
| `skillz info <name>` | Display detailed information about a skill/command (`--show-content`) |
| `skillz update <name>` | Update installed items (**not yet implemented** -- suggests uninstall+install) |
| `skillz create` | Create new skill/command (interactive wizard or AI-assisted via `--prompt`) |
| `skillz config set/get` | Manage configuration |

### Hooks Commands
| Command | Description |
|---------|-------------|
| `skillz hooks list` | List hooks (`--target repo/personal/project`) |
| `skillz hooks install <name>` | Install a hook |
| `skillz hooks uninstall <name>` | Uninstall a hook |
| `skillz hooks info <name>` | Show hook details |
| `skillz hooks search <query>` | Search hooks |
| `skillz hooks create <name>` | Create hook from template or AI (`--event`, `--prompt`) |

### Agents Commands
| Command | Description |
|---------|-------------|
| `skillz agents list` | List agents |
| `skillz agents install <name>` | Install an agent |
| `skillz agents uninstall <name>` | Uninstall an agent |
| `skillz agents info <name>` | Show agent details |
| `skillz agents search <query>` | Search agents |
| `skillz agents create <name>` | Create agent from template or AI (`--model`, `--tools`, `--prompt`) |

### CASCADE Skill Evolution Commands
| Command | Description |
|---------|-------------|
| `skillz harvest` | Extract skill proposals from Claude Code session transcripts |
| `skillz optimize <name>` | Improve skill descriptions via eval feedback (`--all` for batch) |
| `skillz ab-test <name>` | Generate and score multiple description variants |
| `skillz stats` | Track usage frequency, identify unused skills (`--scan`, `--unused`, `--top`) |
| `skillz merge` | Detect overlapping skills and propose merges |

---

## Marketplace/Registry Support

**No.** There is no central marketplace or registry. Skillz uses a local repository model:
- You clone the repo (or any git repo with the right structure)
- Set the repository path via `skillz config set repository /path/to/repo`
- Install from that local repo

The Plugin Guide template (`templates/PLUGIN_GUIDE.md`) mentions "Plugin Registry (Future)" as a planned feature. Distribution is currently via git clone or tarball.

---

## Skill Discovery Features

- **Search:** `skillz search <query>` -- fuzzy matches on skill/command names and descriptions
- **List:** `skillz list` -- shows all available items with type, name, description, and path
- **Info:** `skillz info <name>` -- detailed view with metadata, validation status, file listing, and content preview
- **Stats/scan:** `skillz stats --scan` -- scans Claude Code session transcripts for skill name mentions to estimate usage
- **Harvest:** `skillz harvest` -- analyzes session transcripts to propose new skills from usage patterns

---

## Configuration Options

Config file: `~/.config/skillz/config.yaml`

| Key | Default | Description |
|-----|---------|-------------|
| `default_platform` | `claude` | Default target platform |
| `personal_skills_dir` | `~/.claude/skills` | Personal skills directory |
| `personal_commands_dir` | `~/.claude/commands` | Personal commands directory |
| `project_skills_dir` | `.claude/skills` | Project-level skills directory |
| `project_commands_dir` | `.claude/commands` | Project-level commands directory |
| `default_target` | `personal` | Default install target (personal/project) |
| `repository_path` | (none) | Path to local skillz repository |
| `platforms` | (dict) | Per-platform directory overrides for claude, opencode, codex, gemini |

---

## MCP Server Support

**No.** jkitchin/skillz does not implement or expose an MCP server. It is a CLI tool that copies files to platform-specific directories.

Note: The *other* skillz project on PyPI (by intellectronica) IS an MCP server that exposes skills to MCP clients. These are completely separate projects.

---

## Hooks/Scripts Support

**Yes, comprehensive.** Skillz has first-class hooks support for Claude Code lifecycle events:

- 10 hook events: PreToolUse, PostToolUse, PermissionRequest, UserPromptSubmit, Notification, Stop, SubagentStop, PreCompact, SessionStart, SessionEnd
- Hooks are directories with HOOK.md + executable script (Python or shell)
- Hooks receive JSON on stdin with session context
- Exit codes: 0=allow, 1=error (continue), 2=block (for PreToolUse)
- Tool name matchers via regex
- Configurable timeouts
- Pre-built hooks: lab-notebook, prettier-on-save, black-on-save, protect-secrets, bash-logger, notify-done, ralph-cost-monitor, ralph-safety-check
- AI-assisted hook creation via `--prompt` flag

---

## Skill Dependencies

**Partial.** The Plugin Guide template defines a `plugin.json` manifest format that includes:
```json
{
  "dependencies": {
    "required": ["python>=3.8", "package-name>=1.0.0"],
    "optional": ["optional-package>=2.0.0"]
  }
}
```
However, this is a template/guide for future plugin support -- the CLI does not currently resolve or install dependencies automatically. Skill descriptions can mention dependencies ("Requires: package-name") but it is informational only.

---

## Skill Versioning

**No formal versioning system for individual skills.** The Plugin Guide template supports semantic versioning at the plugin level (via `plugin.json` `version` field), but individual skills do not have version fields in their SKILL.md frontmatter.

The project itself uses semantic versioning (currently 0.1.0).

---

## Skill Sharing/Publishing

**Limited.** Currently distribution is via:
1. **Git repository** -- clone and point skillz at it
2. **Tarball archive** -- package and distribute manually
3. **Plugin manifest** -- `plugin.json` format defined in templates but not yet implemented in CLI

No central registry, no `skillz publish` command, no package upload mechanism.

---

## Custom Skill Directories

**Yes.** Fully configurable:
- `skillz config set repository /path/to/repo` -- set any directory as the skill source
- Per-platform directory overrides in config YAML
- `--target personal` vs `--target project` for installation destination
- Auto-detection: if run inside a directory with `skills/` and `commands/` subdirs, it can auto-configure

---

## Skill Templates

**Yes.** The `templates/` directory includes:
- `SKILL_TEMPLATE.md` -- template for creating new skills
- `COMMAND_TEMPLATE.md` -- template for creating new commands
- `HOOK_TEMPLATE.md` -- template for creating new hooks
- `AGENT_TEMPLATE.md` -- template for creating new agents
- `PLUGIN_GUIDE.md` -- comprehensive guide for creating plugin bundles
- `PLUGIN_TEMPLATE.json` -- template plugin.json manifest

Templates are used by `skillz create`, `skillz hooks create`, and `skillz agents create`.

AI-assisted creation is also available: pass `--prompt "description"` to have Claude CLI generate the content.

---

## Bulk Operations

**Yes, partial:**
- `skillz install --all` -- install all skills and commands from repository
- `skillz optimize --all` -- optimize all skill descriptions in batch
- No `skillz uninstall --all` (must uninstall individually)
- No bulk hooks/agents install

---

## Rollback/Uninstall

**Uninstall: Yes.** `skillz uninstall <name>` removes skills/commands. `skillz hooks uninstall` and `skillz agents uninstall` also available.

**Rollback: No.** No version tracking, no undo, no backup-before-overwrite. The `--force` flag overwrites without backup. The `update` command is not yet implemented.

---

## Bundled Content

### Skills (42+ skills across 12 categories)
- **Academic:** phd-qualifier
- **Programming:** claude-light, emacs-lisp, fairchem, idaes, materials-properties, pycalphad, pymatgen, python-ase, python-best-practices, python-jax, python-multiobjective-optimization, python-optimization, python-plotting, python-regression-statistics, vasp
- **Research:** eln, literature-review, materials-databases, scientific-data-extraction, scientific-reviewer, scientific-workflows
- **Python:** pycse
- **Development:** code-reviewer, ralph-wiggum, tdd, version-control
- **Laboratory:** opentrons (7 sub-skills for lab robotics)
- **Scientific:** design-of-experiments
- **Technical:** troubleshooting
- **Communication:** scientific-writing
- **Creative:** brainstorming, elevenlabs, image-generation, presentations, video-storytelling
- **Productivity:** planning
- **Citation:** citation-verifier

### Commands (17 slash commands)
- Git: /commit, /pr, /changelog
- Code quality: /review, /explain, /refactor, /fix
- Documentation: /doc, /readme (generate-readme), /api
- Research: /cite, /lab-entry, /summarize
- Analysis: /deps, /todo, /find-usage
- Special: /ralph

### Hooks (8 pre-built)
- lab-notebook, prettier-on-save, black-on-save, protect-secrets, bash-logger, notify-done, ralph-cost-monitor, ralph-safety-check

### Agents (5 pre-built)
- code-reviewer, debugger, doc-writer, literature-searcher, test-writer

---

## Notable Features

1. **CASCADE-inspired skill evolution** -- the harvest/optimize/ab-test/stats/merge cycle is unique. It can analyze Claude Code session transcripts to propose new skills, use Claude to generate improved descriptions, and empirically test variants.

2. **AI-assisted creation** -- all four artifact types (skills, commands, hooks, agents) can be generated by passing `--prompt` to the create command, which invokes the `claude` CLI.

3. **Validation system** -- comprehensive validators for all four types (SkillValidator, CommandValidator, HookValidator, AgentValidator) checking frontmatter structure, name format, description length, allowed tools, valid events, etc.

4. **Academic/scientific focus** -- the bundled skills lean heavily toward scientific computing (ASE, VASP, pymatgen, pycalphad, IDAES, Opentrons lab robotics), reflecting the author's background as a CMU professor.

5. **Plugin system (template only)** -- a `plugin.json` manifest format is documented in templates for bundling skills+commands+agents with dependencies and configuration, but the CLI does not yet implement plugin install/management.

6. **Security features** -- path traversal prevention in agent/hook installation, protect-secrets hook to block writes to .env files, AI-generated content preview before saving.

---

## PyPI Name Collision

**Important:** The `skillz` package on PyPI (versions 0.1.0-0.1.14) is a **completely different project** by Eleanor Berger (intellectronica). That project is an MCP server that exposes Claude-style skills to any MCP client. jkitchin/skillz is not published to PyPI and must be installed from source.

---

## Activity Level

- 64 commits from jkitchin, 11 from claude (AI-assisted), 3 from dependabot
- Last commit: 2026-03-07 (10 days ago as of research date)
- No formal releases/tags published on GitHub
- 22 stars, 3 forks, 0 open issues
- CI: GitHub Actions for tests and lint, codecov integration
- Pre-commit hooks configured (ruff check + ruff format)
- Test suite with coverage reporting

---

## Comparison: jkitchin/skillz vs intellectronica/skillz (PyPI)

| Aspect | jkitchin/skillz | intellectronica/skillz |
|--------|----------------|----------------------|
| **Type** | CLI tool (file copier) | MCP server |
| **Install** | `pip install -e .` from source | `pip install skillz` from PyPI |
| **How it works** | Copies SKILL.md files to platform dirs | Exposes skills as MCP tools |
| **Multi-platform** | Claude, OpenCode, Codex, Gemini | Any MCP client |
| **Extras** | Hooks, agents, commands, CASCADE evolution | Script execution |
| **Python** | >=3.10 | >=3.12 |
| **Dependencies** | click, pyyaml, rich | fastmcp, pyyaml |
