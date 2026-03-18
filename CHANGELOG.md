# Changelog

## 0.0.1

Fresh start. All prior versions have been removed — equip is under active development with no backwards compatibility guarantees.

Current features:
- Install SKILL.md files from GitHub repos, git URLs, or local paths
- Auto-detect 18 agents (Claude Code, Codex, Gemini CLI, OpenCode, pi-mono, Amp, Cline, Continue, Cursor, GitHub Copilot, Goose, Kilo Code, Kiro, Pear AI, Roo Code, Sourcegraph Cody, Windsurf, Zed)
- Project-local and global installation scopes
- Cross-machine skill sync via GitHub repos or cloud-synced folders
- `equip survey` and `equip fix` for skill sprawl detection
- `equip outdated` and `equip update` for drift detection
- `includes` file support for referencing skills from other repos
- `--json` flag on all commands
