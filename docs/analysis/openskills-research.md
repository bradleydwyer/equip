# OpenSkills — Research Analysis

**Repository:** https://github.com/numman-ali/openskills
**npm package:** `openskills` (v1.5.0, published 2026-01-17)
**Author:** Numman Ali
**License:** Apache-2.0
**Stars:** 9,037 | **Forks:** 578 | **Open Issues:** 39
**Language:** TypeScript (96.9%)
**Created:** 2025-10-26

---

## Supported Agents/IDEs

1. Claude Code — native compatibility, `.claude/skills/` directory
2. Cursor — via AGENTS.md reading
3. Windsurf — via AGENTS.md reading
4. Aider — via AGENTS.md reading
5. Codex — via AGENTS.md reading
6. Any agent capable of reading AGENTS.md files

Only two directory schemes: `.claude/skills` (default) and `.agent/skills` (universal mode). Does NOT detect or target individual agents — relies on AGENTS.md as a universal format.

## Installation

- `npm i -g openskills` or `npx openskills`
- Requires Node.js >= 20.6.0 and git

## Skill Format

YAML frontmatter with `name` (kebab-case) and `description` (1-2 sentences). Max ~5,000 words.

Directory structure:
```
my-skill/
  SKILL.md              # Required
  references/           # Optional — detailed docs, API specs
  scripts/              # Optional — executable code (not auto-run)
  assets/               # Optional — templates, boilerplate (<10MB)
```

Frontmatter validation is minimal — checks for `---` prefix, extracts fields via regex. For subdirectory skills, the directory name is used as the skill name, not the frontmatter `name`.

## CLI Commands (8)

| Command | Description |
|---------|-------------|
| `install <source>` | Install from GitHub, git URLs, or local paths |
| `sync` | Generate/update AGENTS.md with XML skill listings |
| `list` | Display all installed skills |
| `read <name...>` | Output skill content to stdout (for AI agents at runtime) |
| `update [name...]` | Refresh installed skills from recorded source |
| `manage` | Interactive TUI for removing skills |
| `remove <name>` | Delete a specific skill |
| `--version` / `--help` | Standard flags |

## Source Types

- GitHub shorthand: `owner/repo` (full repo) or `owner/repo/path` (subpath)
- Git URLs: `git@`, `https://`, etc. — cloned with `git clone --depth 1`
- Local paths: absolute, relative, tilde-expanded

## Install Flow

1. Detect source type (local path vs git URL vs GitHub shorthand)
2. Clone to `$HOME/.openskills-temp-{timestamp}` if git source
3. Scan for SKILL.md files (root-level or recursive)
4. Interactive checkbox selection (unless `-y`)
5. Check for conflicts (overwrite prompt)
6. `cpSync` with `dereference: true`
7. Write `.openskills.json` metadata per skill
8. Clean up temp dir

## Metadata (`.openskills.json`)

Written per skill directory. Tracks `source`, `sourceType` (git|local), `repoUrl`, `subpath`, `localPath`, `installedAt`.

## AGENTS.md Sync

Generates Claude Code-compatible XML:
```xml
<skills_system priority="1">
  <usage>...</usage>
  <available_skills>
    <skill>
      <name>...</name>
      <description>...</description>
      <location>...</location>
    </skill>
  </available_skills>
</skills_system>
```

Replacement strategy: looks for `<skills_system>` tag, then `<!-- SKILLS_TABLE_START -->` comments, then appends.

## What It Does NOT Have

- No per-agent directory targeting (only `.claude/skills` or `.agent/skills`)
- No agent detection
- No marketplace or registry
- No MCP server support
- No hooks/lifecycle scripts (scripts/ dir exists but not auto-executed)
- No skill dependencies
- No skill versioning (implicit via git)
- No rollback
- No skill templates or scaffolding
- No quality scoring
- Security: path traversal protection, ReDoS protection

## Key Architectural Decisions

- Agent-agnostic via AGENTS.md as universal discovery format
- Skills loaded on-demand by agents at runtime (progressive disclosure)
- 4 runtime deps only: commander, chalk, ora, @inquirer/prompts
- Symlink support for dev workflows
- CI/CD support via `--yes` flag
