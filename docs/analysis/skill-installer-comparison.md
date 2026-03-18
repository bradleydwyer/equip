# Skill Installer Comparison Matrix

Researched 2026-03-17. Covers the 4 major open-source SKILL.md installer tools.

---

## Overview

| | **AI Agent Skills** | **OpenSkills** | **agent-skills-cli** | **skillz** |
|---|---|---|---|---|
| **GitHub** | skillcreatorai/Ai-Agent-Skills | numman-ali/openskills | Karanjot786/agent-skills-cli | jkitchin/skillz |
| **Stars** | 936 | 9,037 | 56 | 22 |
| **License** | MIT | Apache-2.0 | MIT | (not published) |
| **Language** | TypeScript | TypeScript | TypeScript | Python |
| **Install method** | `npx ai-agent-skills` | `npm i -g openskills` | `npm i -g agent-skills-cli` | `pip install -e .` (clone) |
| **Runtime req** | Node.js | Node.js >= 20.6 | Node.js 18+ | Python 3 |
| **Published to registry** | npm | npm | npm | No (PyPI `skillz` is unrelated) |
| **Last commit** | 2026-03-14 | 2026-03-17 | 2026-02-27 | ~2026-03-07 |
| **Maturity** | Active | Active | Active (solo) | Alpha / academic |

---

## Agent/IDE Support

| Agent | **AI Agent Skills** | **OpenSkills** | **agent-skills-cli** | **skillz** |
|---|:---:|:---:|:---:|:---:|
| **Claude Code** | Y | Y | Y | Y |
| **Cursor** | Y | Y (via AGENTS.md) | Y | N |
| **Codex (OpenAI)** | Y | Y (via AGENTS.md) | Y | Y (configured) |
| **Gemini CLI** | Y | N | Y | Y (configured) |
| **GitHub Copilot** | Y | N | Y | N |
| **VS Code** | Y | N | N | N |
| **Windsurf** | N | Y (via AGENTS.md) | Y | N |
| **Amp** | Y | N | Y | N |
| **Goose** | Y | N | Y | N |
| **OpenCode** | Y | N | Y | Y |
| **Kilo Code / Roo Code** | Y | N | Y | N |
| **Letta** | Y | N | N | N |
| **Aider** | N | Y (via AGENTS.md) | N | N |
| **Cline** | N | N | Y | N |
| **Zed** | N | N | Y | N |
| **Kiro CLI** | N | N | Y | N |
| **Sourcegraph Cody** | N | N | Y | N |
| **Pear AI** | N | N | Y | N |
| **Total agents** | **12** | **6+** (any AGENTS.md) | **45** | **4** |

---

## Skill Format & Structure

| | **AI Agent Skills** | **OpenSkills** | **agent-skills-cli** | **skillz** |
|---|---|---|---|---|
| **Format** | SKILL.md + YAML frontmatter | SKILL.md + YAML frontmatter | SKILL.md + YAML frontmatter | SKILL.md + YAML frontmatter |
| **Max size guidance** | None stated | ~5,000 words | None stated | None stated |
| **`references/` dir** | N | Y | Y | N |
| **`scripts/` dir** | N | Y (not auto-executed) | Y (auto-executable) | N |
| **`assets/` dir** | N | Y (<10MB) | N | N |
| **`rules/` dir** | N | N | Y | N |
| **Artifact types** | Skills only | Skills only | Skills only | Skills, Commands, Hooks, Agents |

---

## CLI Commands

| Capability | **AI Agent Skills** | **OpenSkills** | **agent-skills-cli** | **skillz** |
|---|:---:|:---:|:---:|:---:|
| **install** | Y | Y | Y | Y |
| **uninstall / remove** | Y | Y | Y | Y |
| **list** | Y | Y | Y (`check`) | Y |
| **search** | Y | N | Y (+ fuzzy/FZF) | Y |
| **update** | Y | Y | Y | N (stub) |
| **info / preview** | Y | Y (`read`) | Y | Y |
| **browse (interactive TUI)** | Y | N | Y (`search -i`) | N |
| **init / scaffold** | N | N | Y | Y (`create`) |
| **validate** | N | N | Y | N |
| **score / quality** | N | N | Y (0-100, grades) | N |
| **diff** | N | N | Y | N |
| **compose / merge** | N | N | Y | Y (`merge`) |
| **split** | N | N | Y | N |
| **test** | N | N | Y (10 built-in assertions) | N |
| **benchmark** | N | N | Y | N |
| **watch (auto-sync)** | N | N | Y | N |
| **doctor / diagnose** | N | N | Y (+ conflict detection) | N |
| **audit (security)** | N | N | Y | N |
| **sync to AGENTS.md** | N | Y | N | N |
| **config** | Y | N | N | Y |
| **collections** | Y | N | N | N |
| **context budget** | N | N | Y (token-aware) | N |
| **harvest from transcripts** | N | N | N | Y (CASCADE) |
| **A/B test descriptions** | N | N | N | Y (CASCADE) |
| **optimize via LLM** | N | N | N | Y (CASCADE) |
| **usage stats** | N | N | Y (`insight`) | Y (`stats`) |
| **Total commands** | **~12** | **~8** | **~40+** | **~16** |

---

## Distribution & Discovery

| | **AI Agent Skills** | **OpenSkills** | **agent-skills-cli** | **skillz** |
|---|---|---|---|---|
| **Marketplace** | N (bundled catalog of 48) | N | Y SkillsMP (175K+ skills) | N |
| **Install from GitHub** | Y (`owner/repo`) | Y (`owner/repo`) | Y (`owner/repo`) | N |
| **Install from git URL** | Y (SSH/HTTPS + `#ref`) | Y (SSH/HTTPS) | Y (SSH/HTTPS + `#branch`) | N |
| **Install from local path** | Y | Y | N | Y |
| **Install from npm registry** | N | N | Y (`npm:@scope/pkg`) | N |
| **Private repo support** | Y | Y (SSH/HTTPS auth) | Y (multi-provider auth) | N |
| **Publish / submit** | N (PR-based) | N (share via GitHub) | Y (`submit`, `submit-repo`) | N |
| **Bundled skills** | 48 | 0 (points to anthropics/skills) | 0 | ~20 (scientific focus) |
| **Glob/pattern matching** | N | N | Y (`--skill 'core-*'`) | N |

---

## Advanced Features

| | **AI Agent Skills** | **OpenSkills** | **agent-skills-cli** | **skillz** |
|---|:---:|:---:|:---:|:---:|
| **MCP server support** | N | N | N | N |
| **Hooks / lifecycle scripts** | N | Partial (bundled, not auto-run) | Y (6 languages, 30s timeout) | Y (10 Claude Code events) |
| **Skill dependencies** | N | N | N | N |
| **Skill versioning** | N (global version only) | N (implicit via git) | Partial (lockfile SHA tracking) | N |
| **Lockfile** | N | N | Y (`skills.lock`) | N |
| **Deterministic install** | N | N | Y (`frozen --strict`) | N |
| **Rollback** | N | N | Y (restore from lockfile) | N |
| **Dry-run mode** | N | N | Y (`--dry-run`) | N |
| **Skill templates** | N | N (example only) | Y (`init`, `blueprint`, `craft`) | Y (`create`) |
| **Bulk operations** | Y (all agents default) | Y (multi-skill repos) | Y (extensive) | N |
| **CI/CD mode** | N | Y (`--yes` flag) | Y (`ci`, `frozen --strict`) | N |
| **Symlink support** | N | Y | Y (canonical + symlinks) | N |
| **Quality scoring** | N | N | Y (4-dimension, 0-100) | N |
| **Conflict detection** | N | N | Y (Jaccard similarity) | N |
| **Context budget mgmt** | N | N | Y (token-aware selection) | N |
| **Security scanning** | N | Y (path traversal, ReDoS) | Y (`audit`, script scanning) | N |
| **Telemetry** | N | N | Y (opt-out) | N |
| **Format conversion** | N | N | Y (`convert`) | N |
| **Config file** | N | N | Y (`.skillsrc.json`) | Y |
| **CASCADE (skill evolution)** | N | N | N | Y |
| **Cross-platform (Windows)** | N stated | Y | Y (copy fallback) | N stated |

---

## Summary

| Tool | Best for |
|------|----------|
| **AI Agent Skills** | Quick start — bundled catalog of 48 skills, installs across 12 agents with minimal fuss |
| **OpenSkills** | Most popular (9K stars), cleanest design, focused on SKILL.md standard + AGENTS.md sync |
| **agent-skills-cli** | Power users — 45 agents, marketplace, lockfiles, quality scoring, most CLI commands by far |
| **skillz** | Researchers / academics — CASCADE system for harvesting/optimizing skills, manages 4 artifact types |

**Common gaps across all 4:** No MCP server management, no skill dependency resolution, no formal semantic versioning. The SKILL.md format is the de facto standard across all.

**Gap that equip fills:** None ship as a standalone binary. All require Node.js or Python at runtime.
