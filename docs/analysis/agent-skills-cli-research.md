# agent-skills-cli — Research Analysis

**Repository:** https://github.com/Karanjot786/agent-skills-cli
**npm package:** `agent-skills-cli` (v1.1.7)
**Author:** Karanjot Singh
**License:** MIT
**Stars:** 56 | **Forks:** 3 | **Open Issues:** 1
**Language:** TypeScript (ESM)
**Created:** 2026-01-04
**Website:** https://agentskills.in
**Marketplace:** https://agentskills.in/marketplace

---

## Supported Agents/IDEs (45)

| # | Agent | Project Dir | Global Dir |
|---|-------|-------------|------------|
| 1 | Cursor | `.cursor/skills/` | `~/.cursor/skills/` |
| 2 | Claude Code | `.claude/skills/` | `~/.claude/skills/` |
| 3 | GitHub Copilot | `.github/skills/` | `~/.github/skills/` |
| 4 | Codex (OpenAI) | `.codex/skills/` | `~/.codex/skills/` |
| 5 | Antigravity | `.agent/skills/` | `~/.gemini/antigravity/skills/` |
| 6 | OpenCode | `.opencode/skill/` | `~/.config/opencode/skill/` |
| 7 | Amp | `.agents/skills/` | `~/.config/agents/skills/` |
| 8 | Kilo Code | `.kilocode/skills/` | `~/.kilocode/skills/` |
| 9 | Roo Code | `.roo/skills/` | `~/.roo/skills/` |
| 10 | Goose | `.goose/skills/` | `~/.config/goose/skills/` |
| 11 | Cline | `.cline/skills/` | `~/.cline/skills/` |
| 12 | CodeBuddy | `.codebuddy/skills/` | `~/.codebuddy/skills/` |
| 13 | Command Code | `.commandcode/skills/` | `~/.commandcode/skills/` |
| 14 | Continue | `.continue/skills/` | `~/.continue/skills/` |
| 15 | Crush | `.crush/skills/` | `~/.config/crush/skills/` |
| 16 | Clawdbot | `skills/` | `~/.clawdbot/skills/` |
| 17 | Droid | `.factory/skills/` | `~/.factory/skills/` |
| 18 | Gemini CLI | `.gemini/skills/` | `~/.gemini/skills/` |
| 19 | Kiro CLI | `.kiro/skills/` | `~/.kiro/skills/` |
| 20 | MCPJam | `.mcpjam/skills/` | `~/.mcpjam/skills/` |
| 21 | Mux | `.mux/skills/` | `~/.mux/skills/` |
| 22 | OpenHands | `.openhands/skills/` | `~/.openhands/skills/` |
| 23 | Pi | `.pi/skills/` | `~/.pi/agent/skills/` |
| 24 | Qoder | `.qoder/skills/` | `~/.qoder/skills/` |
| 25 | Qwen Code | `.qwen/skills/` | `~/.qwen/skills/` |
| 26 | Trae | `.trae/skills/` | `~/.trae/skills/` |
| 27 | Windsurf | `.windsurf/skills/` | `~/.codeium/windsurf/skills/` |
| 28 | Zencoder | `.zencoder/skills/` | `~/.zencoder/skills/` |
| 29 | Neovate | `.neovate/skills/` | `~/.neovate/skills/` |
| 30 | Ara | `.ara/skills/` | `~/.ara/skills/` |
| 31 | Aide | `.aide/skills/` | `~/.aide/skills/` |
| 32 | Alex | `.alex/skills/` | `~/.alex/skills/` |
| 33 | BB | `.bb/skills/` | `~/.bb/skills/` |
| 34 | CodeStory | `.codestory/skills/` | `~/.codestory/skills/` |
| 35 | Helix AI | `.helix/skills/` | `~/.helix/skills/` |
| 36 | Meekia | `.meekia/skills/` | `~/.meekia/skills/` |
| 37 | Pear AI | `.pearai/skills/` | `~/.pearai/skills/` |
| 38 | Adal | `.adal/skills/` | `~/.adal/skills/` |
| 39 | Pochi | `.pochi/skills/` | `~/.pochi/skills/` |
| 40 | Sourcegraph Cody | `.sourcegraph/skills/` | `~/.sourcegraph/skills/` |
| 41 | Void AI | `.void/skills/` | `~/.void/skills/` |
| 42 | Zed | `.zed/skills/` | `~/.zed/skills/` |
| 43 | Lingma | `.lingma/skills/` | `~/.lingma/skills/` |
| 44 | Deep Agents | `.deepagents/skills/` | `~/.deepagents/agent/skills/` |
| 45 | Ruler | `.ruler/skills/` | `~/.ruler/skills/` |

Architecture: `AgentAdapter` interface with `BaseAdapter`. Specialized adapters for Cursor/Claude/Copilot, `UniversalAdapter` for all others. Adding a new agent is config-only.

## Installation

- `npm install -g agent-skills-cli` or `npx agent-skills-cli`
- Binary names: `skills` and `agent-skills`
- Requires Node.js 18+

## Skill Format

Standard SKILL.md with YAML frontmatter. Fields: `name` (required, 1-64 chars), `description` (required, 1-1024 chars), `license`, `compatibility`, `metadata` (key-value), `allowedTools` (space-delimited).

Can also include `scripts/`, `rules/`, `references/` directories.

## CLI Commands (~40+)

### Core Commands (11)
- `install <name>` / `add <source>` — Install from marketplace or Git
- `search <query>` — Search and install (multi-select, FZF mode with `-i`)
- `check` — Check installed skills
- `update` — Update skills from source
- `remove` — Remove installed skills (interactive multi-select)
- `score [path]` — Quality scoring (0-100, grades F-A)
- `submit-repo <repo>` — Submit repo for marketplace auto-indexing
- `doctor` — Diagnose issues (`--deep` for conflict detection)
- `init <name>` — Create new skill from template
- `validate <path>` — Validate SKILL.md
- `export` — Export skills to agents

### Power Tools (9)
- `budget -b <tokens>` — Context budget (token-aware skill selection)
- `diff <A> <B>` — Section-aware skill comparison
- `compose <skills...>` — Merge/chain/conditional skill composition
- `test [skills...]` — Quality assertions (10 built-in + custom)
- `frozen` — Deterministic lockfile install (like `npm ci`)
- `sandbox <source>` — Preview quality + conflicts before install
- `watch [dir]` — Auto-sync on file changes
- `split <skill>` — Split large skills into focused sub-skills
- `bench [skills...]` — Benchmark and compare skill quality

### Additional Commands (20+)
suggest, audit, craft, submit, bootstrap, convert, collab, lockspec, forge, mine, recall, grid, capture, trigger, rule, blueprint, ci, track, insight, method, context, info

## Marketplace

SkillsMP at agentskills.in/marketplace — claims 175,000+ skills. Categorized by Development, Testing, DevOps, AI & ML, Security, Data & Analytics, Infrastructure. Free, no API key required.

## Notable Features

- **Quality Scoring:** 4 dimensions (Structure 30%, Clarity 30%, Specificity 30%, Advanced 10%), 0-100 scale, grades F-A, no LLM required
- **Conflict Detection:** Keyword contradiction detection, topic overlap via Jaccard similarity
- **Context Budget Manager:** Token-aware skill selection using 4-signal relevance scoring
- **Lockfile:** `~/.skills/skills.lock` tracks version (git SHA), install/update timestamps
- **Deterministic Install:** `frozen --strict` for CI
- **Symlink-based Installation:** Canonical storage in `~/.skills/` with symlinks to each agent
- **Script Execution:** 6 languages (Python, JS, TS, Bash, Ruby, Perl), 30s timeout, safety scanning
- **Private Git:** SSH, HTTPS with tokens, GitLab, Bitbucket, self-hosted
- **npm Registry:** `skills install npm:@scope/package`
- **Telemetry:** Anonymous, auto-disabled in CI, opt-out via env vars
- **Config file:** `.skillsrc` / `.skillsrc.json`
- **Dry-run mode**, **branch targeting** (`owner/repo#dev`), **glob matching** (`--skill 'core-*'`)

## What It Does NOT Have

- No MCP server support
- No skill dependencies / dependency resolution
- No semantic versioning for skills (metadata.version is informational only)
