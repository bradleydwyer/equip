# Changelog

## 0.1.0

- **Breaking:** Loadout repo ops directory renamed from `ops/` to `.ops/` (hidden directory)
- Auto-migration: existing `ops/` directories are renamed to `.ops/` on first access
- Mark v2 sync plan doc as completed

## 0.0.6

- Rename `equip sync` to `equip agents` (`sync` kept as alias)
- Merge `equip fix` into `equip survey --fix`
- `equip update` now skips up-to-date skills instead of re-installing everything
- Fix pre-existing clippy warnings in init.rs and install.rs

## 0.0.5

- Add `--protocol ssh|https` flag to `equip init` for explicit protocol selection
- Auto-fallback: if SSH clone fails (timeout, host key, permission), automatically retry with HTTPS (and vice versa)
- Use direct `git clone` with constructed URLs instead of `gh repo clone` for better protocol control

## 0.0.4

- Set default git identity (`equip <equip@local>`) in the loadout repo so sync works on machines without global git config
- Auto-reset dirty loadout repo state before pull (recovers from failed syncs)

## 0.0.3

- Fix single-skill repos getting temp directory names (e.g. `equip-1773835822037`) instead of their frontmatter name
- Improve VM integration test: SCP binary for fast iteration, fix credential helper, use `--all` for clean VMs

## 0.0.2

- `equip install` now processes `includes` files found in the source repo, installing referenced skills automatically
- Shared `read_includes` between install and restore (was duplicated in restore only)
- Added VM integration test script (`scripts/test-vm.sh`)

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
