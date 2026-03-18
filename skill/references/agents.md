# Supported Agents

equip knows about 18 agents and their directory conventions:

| ID | Agent | Project dir | Global dir |
|----|-------|-------------|------------|
| `claude` | Claude Code | `.claude/skills` | `~/.claude/skills` |
| `codex` | Codex | `.codex/skills` | `~/.codex/skills` |
| `gemini` | Gemini CLI | `.gemini/skills` | `~/.gemini/skills` |
| `opencode` | OpenCode | `.opencode/skill` | `~/.config/opencode/skill` |
| `pi` | pi-mono | `.agents/skills` | `~/.pi/agent/skills` |
| `amp` | Amp | `.agents/skills` | `~/.config/agents/skills` |
| `cline` | Cline | `.cline/skills` | `~/.cline/skills` |
| `continue` | Continue | `.continue/skills` | `~/.continue/skills` |
| `cursor` | Cursor | `.cursor/skills` | `~/.cursor/skills` |
| `copilot` | GitHub Copilot | `.github/skills` | `~/.github/skills` |
| `goose` | Goose | `.goose/skills` | `~/.config/goose/skills` |
| `kilo` | Kilo Code | `.kilocode/skills` | `~/.kilocode/skills` |
| `kiro` | Kiro | `.kiro/skills` | `~/.kiro/skills` |
| `pearai` | Pear AI | `.pearai/skills` | `~/.pearai/skills` |
| `roo` | Roo Code | `.roo/skills` | `~/.roo/skills` |
| `cody` | Sourcegraph Cody | `.sourcegraph/skills` | `~/.sourcegraph/skills` |
| `windsurf` | Windsurf | `.windsurf/skills` | `~/.codeium/windsurf/skills` |
| `zed` | Zed | `.zed/skills` | `~/.zed/skills` |

Some agents share project-level directories (e.g., Amp and pi-mono both use `.agents/skills`). This is expected and handled correctly.

Agent detection checks for the config directory in `$HOME` (global) or the project root (local). Use `--agent <id>` to target specific agents, `--all` to skip detection and install everywhere.
