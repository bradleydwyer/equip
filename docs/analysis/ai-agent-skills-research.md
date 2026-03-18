# AI Agent Skills - Comprehensive Research

**Date:** 2026-03-17
**Status:** Active
**Repository:** https://github.com/skillcreatorai/Ai-Agent-Skills (redirects from MoizIbnYousaf/Ai-Agent-Skills)
**npm:** `ai-agent-skills` (v1.9.2)
**Website:** https://skillsllm.com/ (separate discovery platform, not the same project)
**Author:** Moiz Ibn Yousaf (SkillCreator.ai)
**License:** MIT
**Stars:** 936
**Forks:** 103
**Created:** 2025-12-17
**Last pushed:** 2026-03-14
**Primary language:** Python (by bytes), JavaScript (CLI), Shell

---

## What It Is

A curated library of 48 agent skills with a universal CLI installer. Positions itself as
"Homebrew for AI Agent Skills" -- a cross-agent package manager that installs skill files
(markdown-based instructions) into the correct directory for whichever coding agent you use.

The core value proposition: skills are vendored/snapshotted from upstream repos (Anthropic,
OpenAI, Composio, wshobson) with explicit provenance tracking, so installs are deterministic
and don't break when upstream reorganizes.

---

## Supported Agents/IDEs (12 targets)

| Agent      | Install Path                       | Scope    |
|------------|------------------------------------|----------|
| Claude Code| `~/.claude/skills/`                | Global   |
| Cursor     | `.cursor/skills/` (project)        | Project  |
| Codex      | `~/.codex/skills/`                 | Global   |
| Amp        | `~/.amp/skills/`                   | Global   |
| VS Code    | `.github/skills/` (project)        | Project  |
| Copilot    | `.github/skills/` (project)        | Project  |
| Gemini CLI | `~/.gemini/skills/`                | Global   |
| Goose      | `~/.config/goose/skills/`          | Global   |
| OpenCode   | `~/.config/opencode/skill/`        | Global   |
| Letta      | `~/.letta/skills/`                 | Global   |
| Kilo Code  | `~/.kilocode/skills/`              | Global   |
| Project    | `.skills/` (portable)              | Project  |

Default behavior: `install` without `--agent` installs to ALL agents.

---

## How Skills Are Defined

Each skill is a directory under `skills/<skill-name>/` containing a single `SKILL.md` file.

### SKILL.md Format

```yaml
---
name: frontend-design
description: Create distinctive, production-grade frontend interfaces...
source: anthropics/skills
license: Apache-2.0
---
```

Followed by free-form markdown sections with instructions, guidelines, anti-patterns, etc.
The YAML frontmatter requires `name` and `description` at minimum. The body is what gets
loaded as context/instructions by the agent.

### skills.json Catalog Entry

Each skill also has a rich entry in `skills.json` with these fields:

- `name` -- lowercase with hyphens (e.g., `frontend-design`)
- `description` -- human-readable purpose
- `category` -- one of: development, document, creative, business, productivity
- `workArea` -- one of: frontend, backend, mobile, docs, testing, workflow, research, design, business
- `branch` -- sub-specialization (e.g., "React", "MCP", "Python", "CI")
- `author` -- original author
- `source` -- upstream repo (e.g., `anthropics/skills`)
- `license` -- Apache-2.0 or MIT
- `path` -- local path in repo
- `tags` -- array of framework/language tags (e.g., `["react", "typescript", "nextjs"]`)
- `featured` -- boolean, highlighted in listings
- `verified` -- boolean, personally validated by curator
- `origin` -- one of: curated, authored, adapted
- `trust` -- one of: listed, reviewed, verified
- `syncMode` -- one of: mirror, snapshot, adapted, authored
- `sourceUrl` -- direct URL to upstream source
- `whyHere` -- editorial note explaining why this skill is in the library
- `lastVerified` -- date of last verification

---

## Installation Method

**npm / npx** -- no global install required:

```bash
npx ai-agent-skills install frontend-design
```

Or install globally:

```bash
npm install -g ai-agent-skills
skills install frontend-design
```

Binary names: `ai-agent-skills` and `skills` (both map to `cli.js`).

Requires Node.js >= 14.16.0.

**Dependencies:** htm, ink, ink-text-input, react (for TUI browser).

---

## CLI Commands

| Command | Aliases | Description |
|---------|---------|-------------|
| `(no command)` | -- | Launch interactive TUI browser (TTY only) |
| `browse` | `b` | Interactive skill browser (TUI) |
| `list` | `ls` | List all available skills |
| `collections` | `catalog` | Show curated collections |
| `install <name>` | `i`, `add` | Install a skill (to ALL agents by default) |
| `install <owner/repo>` | -- | Install from GitHub repository |
| `install <git-url>` | -- | Install from any git URL (SSH/HTTPS) |
| `install ./path` | -- | Install from local filesystem path |
| `uninstall <name>` | `remove`, `rm` | Remove an installed skill |
| `update <name>` | `upgrade` | Update an installed skill |
| `update --all` | -- | Update all installed skills |
| `search <query>` | `s`, `find` | Search by name, description, tags, work area, branch |
| `info <name>` | `show` | Show detailed skill metadata |
| `preview <name>` | -- | Print the full SKILL.md content |
| `config` | -- | Show/edit configuration |
| `version` | -- | Show version number |
| `help` | -- | Show help |

### CLI Flags

| Flag | Short | Description |
|------|-------|-------------|
| `--agent <name>` | `-a` | Target a specific agent |
| `--agents <list>` | -- | Target multiple agents (comma-separated) |
| `--all-agents` | -- | Install to all known agents |
| `--installed` | `-i` | Show only installed skills (with `list`) |
| `--dry-run` | `-n` | Preview changes without applying |
| `--work-area <area>` | `--area` | Filter by work area |
| `--category <cat>` | `-c` | Filter by category |
| `--collection <id>` | -- | Filter by curated collection |
| `--tag <tag>` | `-t` | Filter by framework/language tag |
| `--all` | -- | Apply to all installed (with `update`) |
| `--<agent-name>` | -- | Shorthand for `--agent <name>` (e.g., `--cursor`) |

---

## Marketplace/Registry Support

**Bundled registry:** The 48 skills ship inside the npm package itself (vendored in `skills/` directory). No external registry server.

**External sources supported:**
- GitHub repos: `npx ai-agent-skills install owner/repo` or `owner/repo/skill-name`
- Arbitrary git URLs: SSH (`git@...`) and HTTPS, with optional `#ref` for branch/tag
- Local paths: `./path`, `../path`, `/absolute/path`, `~/path`

**SkillsLLM.com** (https://skillsllm.com/) is a separate, independent discovery platform by Welldanov (not by the same author). It aggregates 1,541 skills from GitHub with 100+ stars, scans them with Semgrep for security, and categorizes them across 10 categories: AI Agents (1,058), MCP Servers (347), CLI Tools (75), IDE Extensions (21), API Integration (16), DevOps (8), Testing (6), Code Generation (4), Data Processing (4), Documentation (2).

---

## Skill Discovery Features

- **Interactive TUI browser** (`browse` command) -- built with Ink/React for terminal
- **Search** with fuzzy matching (Levenshtein distance for "did you mean" suggestions)
- **Work area filtering** -- 9 work areas: frontend, backend, mobile, docs, testing, workflow, research, design, business
- **Category filtering** -- 5 categories: development, document, creative, business, productivity
- **Collection filtering** -- 5 curated collections: my-picks, build-apps, build-systems, test-and-debug, docs-and-research
- **Tag filtering** -- by framework/language tags (react, typescript, python, etc.)
- **Skill info** -- detailed metadata including trust level, origin, sync mode, source URL, editorial "why here" notes
- **Skill preview** -- full SKILL.md content display

---

## Configuration Options

Config file: `~/.agent-skills.json`

| Setting | Default | Description |
|---------|---------|-------------|
| `defaultAgent` | `claude` | Default agent target |
| `agents` | -- | Array of default agents for multi-agent installs |
| `autoUpdate` | `false` | Whether to auto-update |

Set via: `npx ai-agent-skills config --default-agent cursor`
Or: `npx ai-agent-skills config --agents claude,cursor`

---

## MCP Server Support

**Yes, indirectly.** The `mcp-builder` skill is a comprehensive guide for building MCP servers. Several skills reference MCP integrations (e.g., `figma` uses the Figma MCP server, `sentry` uses Sentry API). However, the tool itself does not install or manage MCP servers -- it installs skill files (markdown instructions) that may reference MCP servers.

The npm package keywords include "mcp". Skills like `mcp-builder` teach agents how to create MCP servers using the official SDK and inspector tool.

---

## Hooks/Scripts Support

**No.** The tool does not have a hook or lifecycle script system. It is a straightforward file-copy installer with no pre/post-install hooks, no event system, and no plugin architecture.

---

## Skill Dependencies

**No.** Skills are independent markdown files. There is no dependency resolution, no skill-requires-skill mechanism, and no transitive dependency handling. Each skill is a standalone SKILL.md file.

---

## Skill Versioning

**Partial.** The `skills.json` catalog tracks a global `version` (currently 1.9.2) and per-skill metadata (`lastVerified` date, `syncMode`). However, individual skills do not have their own version numbers. The `syncMode` field distinguishes between:

- `mirror` -- tracks upstream changes
- `snapshot` -- vendored at a specific point in time
- `adapted` -- based on upstream but modified
- `authored` -- original to this library

Update tracking uses `.skill-meta.json` files written alongside installed skills, recording source type (registry/github/git/local) and timestamps.

---

## Skill Sharing/Publishing

**Partial.** You can:
- Submit skills via GitHub PR (following CONTRIBUTING.md and CURATION.md guidelines)
- Install from any GitHub repo or git URL
- Install from local paths

There is no `publish` command, no user accounts, no central registry API. Publishing means getting your PR accepted into the curated catalog or hosting your own repo that others can `install` from.

---

## Custom Skill Directories

**Yes, via the `project` agent target.** Using `--agent project` installs to `.skills/` in the current directory. Additionally, all agent paths are hardcoded but cover 12 distinct locations. There is no `--path` flag for arbitrary custom directories.

---

## Skill Templates

**Yes, indirectly.** The `skill-creator` skill (from Anthropic) is a guide for creating new skills. However, there is no `init` or `create` scaffolding command in the CLI to generate a new skill from a template.

---

## Bulk Operations

**Yes.**
- `install` without `--agent` installs to ALL 12 agent targets by default
- `--agents claude,cursor,codex` for selective multi-agent targeting
- `--all-agents` flag for explicit all-agents targeting
- `update --all` updates all installed skills for an agent
- Installing from a GitHub repo with multiple skills installs them all at once
- `--collection` flag to view/filter groups of skills

---

## Rollback/Uninstall

**Uninstall: Yes.** `uninstall <name>` removes the skill directory. Supports `--dry-run` for preview.
**Rollback: No.** There is no version history, no snapshots, and no way to roll back to a previous version of a skill. Uninstall is a destructive `rmSync` with no undo.

---

## Security Features

- Path traversal attack prevention (validates skill names, blocks `..`, `/`, `\`)
- Symlink following prevention during file copy
- 50 MB maximum skill size limit
- Git URL validation and sanitization (blocks dangerous characters, removes embedded credentials)
- Skill name validation (lowercase alphanumeric + hyphens, max 64 chars)
- GitHub owner/repo name validation
- `execFileSync` used instead of `exec` (prevents shell injection)
- Secure temp directory creation via `mkdtempSync`
- Partial install cleanup on failure

---

## Skill Sources (Provenance)

| Source | Count | License |
|--------|-------|---------|
| Anthropic (anthropics/skills) | 13 | Apache-2.0 |
| OpenAI (openai/skills) | 7 | Apache-2.0 / MIT |
| wshobson/agents | 7 | MIT |
| ComposioHQ/awesome-claude-skills | 15 | Apache-2.0 |
| Original (MoizIbnYousaf) | 5 | MIT |
| Adapted (community) | 1 | MIT |

---

## Trust/Curation System

Three-tier trust model:
- **Listed** -- basic inclusion, no strong endorsement
- **Reviewed** -- editorial backing
- **Verified** -- personally validated by curator

Four origin types:
- **Curated** -- selected from upstream repos
- **Authored** -- original to this library
- **Adapted** -- based on external source but modified
- **Mirror** vs **Snapshot** -- mirrors track upstream; snapshots are vendored at a point in time

Each skill has a `whyHere` field explaining the editorial rationale for inclusion.

---

## Version History (Key Releases)

| Version | Date | Highlights |
|---------|------|------------|
| 1.0.0 | 2025-12-17 | Initial release, 20 skills, npx installer |
| 1.1.0 | 2025-12-20 | `--dry-run`, config file, update notifications, security (path traversal, 50MB limit) |
| 1.2.0 | 2025-12-20 | Interactive `browse` TUI, GitHub repo install, local path install |
| 1.6.0 | 2025-12-26 | Multi-agent operations (`--agents` flag) |
| 1.8.0 | 2026-01-12 | 11 agents supported, Gemini CLI added |
| 1.9.0 | 2026-01-16 | Vercel/Expo skills, framework tags |
| 1.9.1 | 2026-01-17 | Git URL install (SSH/HTTPS), improved validation |
| 1.9.2 | 2026-01-23 | Added `best-practices` skill |

22 total npm releases published.

---

## Notable Features Summary

| Feature | Supported? | Notes |
|---------|-----------|-------|
| Multi-agent install | Yes | 12 agents, defaults to ALL |
| Interactive TUI browser | Yes | Ink/React-based terminal UI |
| Install from registry | Yes | 48 bundled skills |
| Install from GitHub | Yes | owner/repo or owner/repo/skill |
| Install from git URL | Yes | SSH, HTTPS, with #ref support |
| Install from local path | Yes | ./path, ~/path, /absolute |
| Uninstall | Yes | With --dry-run preview |
| Update (single/all) | Yes | Source-aware (registry/github/git/local) |
| Dry-run mode | Yes | For install, uninstall, update |
| Search with fuzzy match | Yes | Levenshtein "did you mean" |
| Collections/categories | Yes | 5 collections, 5 categories, 9 work areas |
| Tag filtering | Yes | Framework/language tags |
| Config file | Yes | ~/.agent-skills.json |
| Skill metadata tracking | Yes | .skill-meta.json per install |
| Provenance tracking | Yes | Source, trust, origin, syncMode, whyHere |
| Security validation | Yes | Path traversal, injection, size limits |
| MCP server management | No | Has MCP-related skills, but doesn't manage MCP servers |
| Hooks/scripts | No | No lifecycle hooks |
| Skill dependencies | No | Skills are independent |
| Individual skill versions | No | Global version only |
| Publish command | No | PR-based contribution only |
| Rollback | No | No version history |
| Custom directories | Partial | Via `--agent project` (.skills/) only |
| Skill templates/scaffolding | No | No `init` or `create` command |
| Bulk install | Yes | Multi-agent, multi-skill from repos |
| Legacy collection aliases | Yes | 7 aliased collection names |

---

## Related Projects

- **SkillsLLM.com** -- Independent discovery marketplace by Welldanov. Aggregates 1,541 GitHub skills with 100+ stars, security scanning via Semgrep. Not affiliated with this npm package.
- **Anthropic Skills** (anthropics/skills) -- Upstream source for 13 skills
- **OpenAI Skills** (openai/skills) -- Upstream source for 7 skills
- **ComposioHQ/awesome-claude-skills** -- Upstream source for 15 skills
- **wshobson/agents** -- Upstream source for 7 skills

---

## Assessment

**Strengths:**
- Clean CLI UX with good defaults (install to all agents)
- Strong provenance tracking with editorial rationale
- Security-conscious implementation (no shell injection, path traversal prevention)
- Practical curation approach -- 48 vetted skills, not a sprawling dump
- Multiple install sources (registry, GitHub, git, local)
- Interactive TUI for browsing

**Limitations:**
- No skill dependency management
- No individual skill versioning
- No rollback capability
- No publish/share workflow beyond GitHub PRs
- No hook/lifecycle system
- No central registry API (everything bundled in npm package)
- Small catalog (48 skills) compared to ecosystem size
- Skills are just markdown files -- no executable components, no tool installation

**Activity level:** Active. Last commit 2026-03-14, 22 npm releases over 3 months, 936 stars. Single maintainer project.
