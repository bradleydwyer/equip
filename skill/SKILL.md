---
name: equip
description: Manage SKILL.md files across AI coding agents using the equip CLI. Use this skill whenever the user wants to install, uninstall, list, update, check for outdated skills, survey, fix, sync, export, restore, configure, or check status of skills from GitHub repos, git URLs, or local paths. Also trigger when the user asks about managing skills across multiple agents or machines, wants to see what skills are installed, needs to find skill sprawl or duplicates, wants to check if skills are up to date, or needs to sync skills to a new machine. If the user mentions "equip" by name, this skill definitely applies.
---

# equip: Cross-Agent Skill Manager

> **Warning:** equip is under active development (v0.1.2). Expect breaking changes — there are no backwards compatibility guarantees yet.

`equip` is a CLI that installs SKILL.md files to the correct directory for every AI coding agent on the user's machine. It auto-detects which agents are present and copies skills to all of them in one command. Skills install globally by default.

## Installation

If `equip` is not installed, tell the user to run:

```bash
brew install bradleydwyer/tap/equip
```

Verify with `equip --version`.

## Supported Agents

equip supports 18 agents (Claude Code, Codex, Gemini CLI, OpenCode, pi-mono, Amp, Cline, Continue, Cursor, GitHub Copilot, Goose, Kilo Code, Kiro, Pear AI, Roo Code, Sourcegraph Cody, Windsurf, Zed). See `references/agents.md` for the full table of agent IDs and directory paths.

By default, `equip install` detects which agents are present and installs to all of them globally. Use `--agent` to target specific ones, `--all` to install everywhere regardless of detection, or `--local` for project-local scope.

## Scope: Global vs Project-Local

- **Global** (default): Skills install to `~/.claude/skills/`, `~/.cursor/skills/`, etc. Available everywhere.
- **Project-local** (`--local`): Skills install to `.claude/skills/`, `.cursor/skills/`, etc. in the current working directory. Scoped to that project.

## Commands

### Install a skill

```bash
equip install anthropics/skills            # all skills from a GitHub repo (global)
equip install anthropics/skills/skills/pdf  # specific skill via subpath
equip install https://github.com/user/repo.git  # git URL
equip install ./my-skill                   # local path
equip install ./my-skill --local           # install to project-local scope
equip install ./my-skill --agent claude,cursor  # specific agents
equip install ./my-skill --all             # all 18 agents
```

### Remove a skill

```bash
equip remove my-skill                      # remove globally (default)
equip uninstall my-skill                   # alias for remove
equip remove my-skill --local              # remove from project
equip remove my-skill --agent claude       # remove from specific agent
```

### List installed skills

```bash
equip list                # global skills (default)
equip list --short        # names only, no descriptions
equip list --local        # project-local skills
equip list --json         # machine-readable output (includes source field)
```

### Update skills from their original source

```bash
equip update              # update all installed skills
equip update my-skill     # update a specific skill
```

Skips up-to-date skills. Warns if local changes will be overwritten.

### Check for outdated skills

```bash
equip outdated            # check all skills
equip outdated my-skill   # check a specific skill
equip outdated --json     # machine-readable output
```

Detects two types of drift:
- **Upstream changed**: source repo or local source dir has newer content than what's installed
- **Locally modified**: user edited the installed copy since install

Works for both git and local sources. For git, uses `git ls-remote` (fast, no clone). Skills installed before outdated tracking was added show as "unknown" — reinstall to enable tracking.

### Survey for issues

Scan for skill sprawl, duplicates, version mismatches, and unmanaged skills:

```bash
equip survey              # global skills (default)
equip survey --local      # project-local skills
equip survey --path ~/dev # scan all projects under a directory
equip survey --json       # machine-readable output
```

Detects: coverage gaps, content mismatches, source mismatches, unmanaged skills, and orphaned skills.

If `projects_path` is configured (see `equip config`), `equip survey` without `--path` automatically scans that directory.

### Fix issues

Add `--fix` to survey to interactively resolve issues:

```bash
equip survey --fix        # survey then fix interactively
equip survey --fix --json # output a fix plan as JSON
```

Actions: spread (copy to missing agents), align (sync mismatched versions), adopt (write .equip.json for unmanaged skills), prune (remove from undetected agents).

### Cross-machine sync

Sync skills across machines using a GitHub repo or cloud-synced folder. The workflow:

1. **Once per machine:** `equip init` links equip to a sync backend (GitHub repo or file path)
2. **Day-to-day:** `equip install` and `equip remove` auto-sync — each operation writes to the backend so the manifest stays current without manual exports
3. **New machine:** `equip init` + `equip restore` installs everything from the backend

The sync repo stores both an append-only operation log and actual skill content, so restore works offline without needing the original upstream sources.

```bash
# Link to a sync backend (once per machine)
equip init                                # defaults to <gh-user>/equip-loadout (public, created if needed)
equip init user/custom-repo               # use a specific GitHub repo
equip init --path ~/iCloud/equip/         # file path (iCloud, Dropbox, NAS)

# Export current skills (first time, or manual re-sync)
equip export                              # push to linked backend
equip export --output skills.json         # export to file instead

# Restore on a new machine
equip init                                # link to equip-loadout repo
equip restore                             # install all skills from backend
equip restore --from skills.json          # restore from file
equip restore --dry-run                   # preview without installing

# Check sync state
equip status                              # show synced/missing/untracked
equip status --json
```

### Includes

An `includes` file in the sync repo root references skills from other repos. One source per line, `#` comments, blank lines ignored:

```
bradleydwyer/available
bradleydwyer/sloppy/skill
anthropics/skills/skills/pdf
```

`equip restore` processes includes after restoring local skills. This lets you compose your equip-loadout from multiple repos without copying everything into one place.

### Generate AGENTS.md

```bash
equip agents                      # writes AGENTS.md from project-local skills
equip agents --output SKILLS.md   # custom output path
```

### Configuration

```bash
equip config                              # show all settings
equip config projects_path ~/dev          # set default survey scan path
equip config projects_path unset          # clear a setting
```

## JSON Output

Every command supports `--json` for machine-readable output.

### Install

```bash
equip install ./my-skill --agent claude --json
```
```json
{
  "action": "install",
  "source": "./my-skill",
  "global": true,
  "skills": [
    {
      "name": "my-skill",
      "description": "What the skill does",
      "agents": ["Claude Code"],
      "paths": ["/Users/you/.claude/skills/my-skill"]
    }
  ]
}
```

### List

```bash
equip list --json
```
```json
[
  {
    "name": "my-skill",
    "description": "What the skill does",
    "agents": ["Claude Code", "Cursor"],
    "global": true,
    "source": "anthropics/skills/my-skill"
  }
]
```

### Outdated

```bash
equip outdated --json
```
```json
{
  "action": "outdated",
  "global": true,
  "skills": [
    {
      "name": "my-skill",
      "source": "owner/repo",
      "source_type": "git",
      "status": "up_to_date",
      "installed_commit": "abc123...",
      "remote_commit": "abc123...",
      "installed_hash": "a1b2c3d4e5f60708",
      "current_hash": "a1b2c3d4e5f60708"
    }
  ]
}
```

Status values: `up_to_date`, `upstream_changed`, `locally_modified`, `both`, `unknown`, `local_source`, `check_failed`.

### Survey

```bash
equip survey --json
```
```json
{
  "action": "survey",
  "skills": [{"name": "my-skill", "instances": [...]}],
  "issues": [
    {"skill": "my-skill", "kind": "coverage_gap", "detail": "..."}
  ]
}
```

### Fix

```bash
equip survey --fix --json
```
```json
{
  "action": "fix",
  "plan": [
    {"action": "spread", "skill": "my-skill", "target_agents": ["codex"]},
    {"action": "adopt", "skill": "other", "agent": "Claude Code", "path": "..."}
  ]
}
```

### Status

```bash
equip status --json
```
```json
{
  "synced": ["my-skill", "other"],
  "missing": [],
  "untracked": ["local-only"]
}
```

### Restore

```bash
equip restore --json
```
```json
{
  "action": "restore",
  "restored": 3,
  "skipped": 1,
  "failed": 0,
  "skills": [...]
}
```

## Common Errors

| Error | Cause | Fix |
|-------|-------|-----|
| "No AI coding agents detected" | No agent config dirs found in `$HOME` | Use `--agent <id>` or `--all` to target agents explicitly |
| "Subpath not found in repository" | Wrong path in GitHub shorthand | Check the repo structure — skills may be nested (e.g., `anthropics/skills/skills/pdf`) |
| "git clone failed" / "gh repo clone failed" | Repo doesn't exist or auth issue | Verify the repo exists, check `gh auth status` |
| "No sync backend configured" | Running export/restore/status without `equip init` | Run `equip init` first, or use `--output`/`--from` for file-based export/restore |
| "gh CLI is required" | `equip init` with GitHub repo but `gh` not installed | Install with `brew install gh` and authenticate with `gh auth login` |

## SKILL.md Format

Skills are directories containing a `SKILL.md` file with YAML frontmatter:

```markdown
---
name: my-skill
description: What this skill does and when to use it
---

# My Skill

Instructions for the AI agent go here.
```

Optional subdirectories: `references/` (docs loaded on demand), `scripts/` (executable code), `assets/` (templates, boilerplate).
