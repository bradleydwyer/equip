# equip v2: Cross-Machine Skill Sync

## Context

equip installs skills locally per-machine. Users switching machines must re-run every `equip install`. The goal: link equip to a sync backend (GitHub repo, file path, or future service) that keeps a manifest in sync. Install/remove auto-sync. Restore pulls and installs.

## Breaking Change: Global Default

All commands default to global scope. `--global/-g` replaced by `--local/-l`.

## User Flows

```bash
# GitHub repo backend
equip init user/equip-config
# => Created private repo user/equip-config
# => Linked equip to github.com/user/equip-config

# File path backend (covers iCloud, Dropbox, NAS, etc.)
equip init --path ~/Library/Mobile\ Documents/com~apple~CloudDocs/equip/

# Normal usage — manifest auto-syncs after install/remove
equip install anthropics/skills/pdf
# => Installed pdf to [Claude Code, Cursor]
# => Synced manifest

equip remove pdf
# => Removed pdf
# => Synced manifest

# New machine — restore from linked backend
equip init user/equip-config    # or equip init --path ...
equip restore
# => Restored 5 skills to 3 agents

# Check sync state
equip status
# => 5 synced, 0 missing, 1 untracked

# Manual file export (no backend needed)
equip export --output skills.json
equip restore --from skills.json
```

---

## 1. Flip `--global` to `--local`

**Files:** `src/main.rs`, all command files

Replace `#[arg(short, long)] global: bool` with `#[arg(short, long)] local: bool` on install, remove, list, update, survey, fix. Invert logic: `let global = !local`.

## 2. Operation Log Sync Model

Instead of a single manifest file (which creates sync conflicts), equip uses an **append-only operation log**. Each install/remove writes a new file. No file is ever modified or deleted. State is computed by replaying all ops.

**Ops directory structure:**
```
ops/
  20260315T100000Z-add-pdf.json
  20260316T120000Z-add-commit.json
  20260317T140000Z-remove-pdf.json
```

**Op file format:**
```json
{
  "op": "add",
  "skill": "pdf",
  "source": "anthropics/skills/pdf",
  "description": "Convert content to PDF",
  "ts": "2026-03-15T10:00:00Z"
}
```

For remove ops, `source` and `description` are omitted.

**Computing state:** Scan all op files, group by skill name, latest timestamp wins. If latest op is "add" → skill is active. If "remove" → skill is gone.

**Where ops live:**
- Git backend: `~/.equip/repo/ops/` (committed + pushed)
- File backend: `{path}/ops/` (synced by iCloud/Dropbox)

**Zero conflicts:** No file is ever modified. iCloud/Dropbox sync new files without issues. Git never has merge conflicts (only new files added).

## 2a. Sync Backend

**New module:** `src/sync.rs`

```rust
pub enum SyncBackend {
    Git { repo: String, repo_url: String },
    File { path: PathBuf },
}

impl SyncBackend {
    /// Returns path to ops directory
    pub fn ops_dir(&self) -> PathBuf

    /// Pull latest ops (git pull for Git backend, no-op for File backend)
    pub fn pull(&self) -> Result<(), String>

    /// Push new ops (git add + commit + push for Git, no-op for File)
    pub fn push(&self) -> Result<(), String>

    /// Write a new op file to the ops directory
    pub fn write_op(&self, op: &Op) -> Result<(), String>
}
```

**Git backend:** `pull()` = `git pull --rebase`, `push()` = `git add ops/ && git commit && git push`
**File backend:** `pull()` = no-op (iCloud syncs automatically), `push()` = no-op

## 3. Config: `~/.equip/config.json`

**New module:** `src/config.rs`

```json
{
  "backend": "git",
  "repo": "user/equip-config",
  "repo_url": "https://github.com/user/equip-config.git"
}
```

or:

```json
{
  "backend": "file",
  "path": "/Users/brad/Library/Mobile Documents/com~apple~CloudDocs/equip"
}
```

Functions:
- `read() -> Option<EquipConfig>`
- `write(config)`
- `backend() -> Option<SyncBackend>` — constructs backend from config
- `equip_dir() -> PathBuf` — `~/.equip/`
- `repo_dir() -> PathBuf` — `~/.equip/repo/`

## 4. `equip init`

**File:** `src/commands/init.rs` (new)

```
equip init user/equip-config          # GitHub repo
equip init --path ~/iCloud/equip/     # file path
```

**GitHub flow:**
1. Parse source as GitHub shorthand
2. Create `~/.equip/` directory
3. Check if `gh` CLI is available
4. Try `git clone` — if repo doesn't exist AND `gh` is available:
   - `gh repo create user/equip-config --private --clone -- ~/.equip/repo/`
   - `git -C ~/.equip/repo/ commit --allow-empty -m "init equip manifest"`
   - `git -C ~/.equip/repo/ push`
5. If repo doesn't exist and no `gh`:
   - Error: "Repository not found. Install gh CLI (`brew install gh`) or create it manually."
6. Write config.json with `backend: "git"`
7. If ops exist in repo: "Found N skills in sync log. Run `equip restore` to install."

**File path flow:**
1. Create the target directory and `ops/` subdirectory if needed
2. Write config.json with `backend: "file"` and the path
3. If ops exist at path: "Found N skills in sync log. Run `equip restore` to install."

## 5. Auto-sync on install/remove

**Files:** `src/commands/install.rs`, `src/commands/remove.rs`

After a successful global install or remove:
1. Load backend from config (`config::backend()`)
2. If backend exists:
   - Write a new op file (e.g., `20260318T120000Z-add-pdf.json` or `20260318T120000Z-remove-pdf.json`)
   - Call `backend.push()` (git commit + push; no-op for file backend)
   - Print: "Synced"
3. If push fails: warn but don't fail the install/remove
4. If no backend configured: do nothing silently

## 6. `equip export`

**File:** `src/commands/export.rs` (new)

```
equip export                # write ops for all installed skills to linked backend
equip export --output X     # write computed state as JSON to file
equip export --json         # print computed state as JSON to stdout
```

Repo/file-backend mode: scans all installed global skills and writes add ops for any not already in the log. Then pushes (git) or no-op (file).

File/stdout mode: computes current state from installed skills and outputs flat JSON:
```json
[
  {
    "name": "pdf",
    "source": "anthropics/skills/pdf",
    "description": "Convert content to PDF"
  }
]
```

This flat format is what `equip restore --from` reads.

## 7. `equip restore`

**File:** `src/commands/restore.rs` (new)

```
equip restore               # pull from linked backend
equip restore --from X      # read from file
equip restore --from -      # read from stdin
equip restore --dry-run     # show what would be installed
equip restore --json        # JSON output
```

Steps (backend mode):
1. `backend.pull()` (git pull / no-op for file)
2. `ops::compute_state(backend.ops_dir())` — replay ops to get active skills
3. For each active skill with non-null source: call `install::run()`
4. Skip entries with `source: null` — warn per skill
5. Report: "Restored N, skipped M, failed K"

Steps (--from file mode):
1. Read flat JSON array from file/stdin
2. For each entry with non-null source: call `install::run()`
3. Same skip/report logic

## 8. Enrich `list --json` with source

**File:** `src/commands/list.rs`

Add `source: Option<String>` to `InstalledSkill`. Read `.equip.json` in collection loop. Include in JSON output.

## 9. Ops module: `src/ops.rs`

```rust
#[derive(Serialize, Deserialize)]
pub struct Op {
    pub op: OpKind,         // "add" or "remove"
    pub skill: String,
    pub source: Option<String>,
    pub description: Option<String>,
    pub ts: String,         // ISO 8601
}

#[derive(Serialize, Deserialize)]
pub enum OpKind { Add, Remove }

/// Computed state from replaying ops
pub struct SkillState {
    pub source: Option<String>,
    pub description: String,
}

/// Read all op files from a directory, return computed state
pub fn compute_state(ops_dir: &Path) -> Result<BTreeMap<String, SkillState>>

/// Write a new op file to ops directory
pub fn write_op(ops_dir: &Path, op: &Op) -> Result<()>

/// Build ops from currently installed global skills (for initial export)
pub fn build_ops_from_installed() -> Result<Vec<Op>>
```

**State computation:** scan `ops/` dir, parse all JSON files, sort by `ts`, group by skill name, take latest op per skill. Active = latest is "add".

---

## Files to create/modify

| File | Action |
|------|--------|
| `src/config.rs` | **Create** — EquipConfig, backend loading |
| `src/sync.rs` | **Create** — SyncBackend enum, pull/push/write_op |
| `src/ops.rs` | **Create** — Op types, compute_state, write_op, build_ops_from_installed |
| `src/commands/init.rs` | **Create** — `equip init` with git + file backends |
| `src/commands/export.rs` | **Create** — `equip export` |
| `src/commands/restore.rs` | **Create** — `equip restore` |
| `src/commands/status.rs` | **Create** — `equip status` |
| `src/main.rs` | Modify — add commands, flip global→local |
| `src/commands/mod.rs` | Modify — add modules |
| `src/lib.rs` | Modify — add `pub mod config; pub mod sync; pub mod ops;` |
| `src/commands/install.rs` | Modify — flip global→local, add auto-sync |
| `src/commands/remove.rs` | Modify — flip global→local, add auto-sync |
| `src/commands/list.rs` | Modify — flip global→local, add source to JSON |
| `src/commands/update.rs` | Modify — flip global→local |
| `src/commands/survey.rs` | Modify — flip global→local |
| `src/commands/fix.rs` | Modify — flip global→local |
| `README.md` | Modify — document new commands + defaults |

## Existing code to reuse

- `install::run()` (`src/commands/install.rs:10`) — restore calls this per skill
- `SkillSource::parse()` (`src/source.rs`) — init reuses for GitHub shorthand
- `metadata::SkillMetadata::read()` (`src/metadata.rs:27`) — read source from .equip.json
- `metadata::now_iso8601()` (`src/metadata.rs:36`) — timestamps
- `agents::AGENTS` + `agents::skill_dir()` (`src/agents.rs`) — scan installed skills
- `skill::read_skill()` (`src/skill.rs`) — parse SKILL.md frontmatter
- Git clone pattern from `install.rs:163`

## Implementation order

1. Flip `--global` to `--local` across all commands (touches everything, do first)
2. `src/config.rs` — config read/write
3. `src/sync.rs` — SyncBackend with Git + File variants
4. `src/ops.rs` — Op types, compute_state, write_op
5. `src/commands/list.rs` — add source to JSON
6. `src/commands/init.rs` — init command (both backends)
7. `src/commands/export.rs` — export command
8. `src/commands/restore.rs` — restore command
9. `src/commands/install.rs` + `remove.rs` — add auto-sync
10. `src/commands/status.rs` — status command
11. Wire up in `main.rs`, `mod.rs`, `lib.rs`
12. Tests
13. README

## 10. `equip status`

**File:** `src/commands/status.rs` (new)

```
equip status
```

Shows sync state: what's installed locally vs what's in the manifest.

Steps:
1. Load backend. If none: "No sync backend configured. Run `equip init` to set one up."
2. Pull latest manifest (don't modify anything)
3. Build manifest from currently installed global skills
4. Compare the two:
   - **In manifest, not installed locally** — "Missing: pdf (anthropics/skills/pdf)"
   - **Installed locally, not in manifest** — "Untracked: my-skill"
   - **Both match** — "Synced: pdf"
5. Summary: "N synced, M missing, K untracked"

Output supports `--json` for machine-readable format.

## Future (not in v2)

- `equip init --service` — centralized equip cloud backend
- Version pinning (git refs / content hashes)
- Manifest diffing / conflict resolution

## Tests

All tests use `tempdir()` for isolation and `env("HOME", ...)` to avoid touching real config. Tests follow existing patterns in `tests/cli.rs`.

### Unit tests: `src/ops.rs` (mod tests)

```
ops_write_creates_file
  - write_op to a tempdir, verify file exists with correct name pattern

ops_compute_state_single_add
  - write one add op, compute_state returns that skill as active

ops_compute_state_add_then_remove
  - write add then remove for same skill, compute_state returns empty

ops_compute_state_remove_then_readd
  - write add, remove, add again (later ts), compute_state returns active

ops_compute_state_multiple_skills
  - write ops for 3 different skills, verify all 3 in state

ops_compute_state_latest_wins
  - two adds for same skill with different sources, latest source wins

ops_compute_state_empty_dir
  - empty ops dir returns empty state

ops_compute_state_ignores_non_json
  - place a .txt file in ops dir, verify it's ignored

ops_op_serialization_roundtrip
  - serialize Op to JSON and back, verify fields match
```

### Unit tests: `src/config.rs` (mod tests)

```
config_write_and_read_git
  - write git config, read back, verify fields

config_write_and_read_file
  - write file-path config, read back, verify fields

config_read_missing_returns_none
  - read from nonexistent path, returns None

config_equip_dir_uses_home
  - verify equip_dir() returns ~/.equip/

config_backend_from_git_config
  - write git config, call backend(), verify SyncBackend::Git

config_backend_from_file_config
  - write file config, call backend(), verify SyncBackend::File

config_backend_none_when_no_config
  - no config file, backend() returns None
```

### Unit tests: `src/sync.rs` (mod tests)

```
file_backend_ops_dir
  - verify File backend returns {path}/ops/

git_backend_ops_dir
  - verify Git backend returns ~/.equip/repo/ops/

file_backend_pull_is_noop
  - File backend pull() succeeds without error

file_backend_push_is_noop
  - File backend push() succeeds without error
```

### Integration tests: `tests/cli.rs` — Global Default

```
install_defaults_to_global
  - equip install fixture --agent claude (no --local)
  - verify skill in HOME/.claude/skills/ not project dir

install_local_flag_uses_project
  - equip install fixture --agent claude --local
  - verify skill in project/.claude/skills/ not home

list_defaults_to_global
  - install global skill, run equip list, verify it shows up

list_local_flag
  - install local skill, run equip list --local, verify it shows up

remove_defaults_to_global
  - install global, equip remove name, verify removed from HOME

help_shows_new_commands
  - equip --help shows init, export, restore, status
```

### Integration tests: `tests/cli.rs` — list --json source field

```
list_json_includes_source
  - install from local path, list --json
  - verify JSON includes "source" field with the local path

list_json_source_null_for_unmanaged
  - manually place a skill without .equip.json
  - list --json, verify "source" is null
```

### Integration tests: `tests/cli.rs` — Init

```
init_file_backend_creates_config
  - equip init --path {tempdir}
  - verify ~/.equip/config.json exists with backend: "file"

init_file_backend_creates_ops_dir
  - equip init --path {tempdir}
  - verify {tempdir}/ops/ directory exists

init_file_backend_detects_existing_ops
  - pre-create ops dir with an add op
  - equip init --path {tempdir}
  - verify output mentions "Found N skills"

init_overwrites_previous_config
  - equip init --path {dir1}
  - equip init --path {dir2}
  - verify config points to dir2

init_missing_source_errors
  - equip init (no args)
  - verify error about missing source
```

### Integration tests: `tests/cli.rs` — Export

```
export_to_file_backend
  - init with file backend
  - install a skill globally
  - equip export
  - verify ops file created in backend ops dir

export_output_flag_writes_json
  - install a skill globally
  - equip export --output {tempfile}
  - verify file contains JSON array with skill name and source

export_json_flag_prints_stdout
  - install a skill globally
  - equip export --json
  - verify stdout is valid JSON with skill entries

export_no_backend_with_output_flag_works
  - no init (no backend configured)
  - equip export --output {tempfile}
  - verify file written successfully

export_no_backend_no_output_errors
  - no init
  - equip export
  - verify error message about no backend

export_skips_already_tracked_skills
  - init, install skill, export (creates op)
  - export again without changes
  - verify no duplicate op files

export_includes_unmanaged_with_null_source
  - manually place skill without .equip.json
  - equip export --json
  - verify skill appears with source: null
```

### Integration tests: `tests/cli.rs` — Restore

```
restore_from_file_backend
  - init with file backend
  - write add op files for 2 skills (using fixture paths as sources)
  - equip restore
  - verify both skills installed in HOME

restore_from_file_flag
  - create JSON file: [{"name": "valid-skill", "source": "{fixture_path}"}]
  - equip restore --from {file}
  - verify skill installed

restore_from_stdin
  - pipe JSON to equip restore --from -
  - verify skill installed

restore_dry_run_no_side_effects
  - init with file backend, write ops
  - equip restore --dry-run
  - verify no skills actually installed
  - verify output lists what would be installed

restore_skips_null_source
  - write op with source: null
  - equip restore
  - verify warning printed, skill not installed

restore_skips_removed_skills
  - write add op then remove op for same skill
  - equip restore
  - verify skill NOT installed

restore_no_backend_no_from_errors
  - no init, equip restore (no --from)
  - verify error about no backend

restore_json_output
  - equip restore --from {file} --json
  - verify JSON output with restored/skipped/failed counts

restore_already_installed_reinstalls
  - install skill, write op, restore
  - verify skill still present (reinstalled / updated)
```

### Integration tests: `tests/cli.rs` — Status

```
status_all_synced
  - init file backend, install skill, export
  - equip status
  - verify "synced" count matches

status_missing_skill
  - init file backend, write add op for skill not installed locally
  - equip status
  - verify "missing" count = 1, skill name shown

status_untracked_skill
  - init file backend (empty ops)
  - install skill globally
  - equip status
  - verify "untracked" count = 1

status_no_backend_configured
  - equip status (no init)
  - verify message about running equip init

status_json_output
  - equip status --json
  - verify JSON with synced/missing/untracked arrays

status_ignores_removed_in_ops
  - write add then remove op
  - install that skill locally
  - equip status
  - verify skill shows as "untracked" (removed in ops but present locally)
```

### Integration tests: `tests/cli.rs` — Auto-sync

```
install_auto_syncs_when_backend_exists
  - init with file backend
  - equip install fixture --agent claude
  - verify new add op file created in backend ops dir

remove_auto_syncs_when_backend_exists
  - init with file backend, install skill
  - equip remove skill-name
  - verify new remove op file created in backend ops dir

install_no_sync_when_no_backend
  - no init
  - equip install fixture --agent claude
  - verify no ops dir or files created

install_local_does_not_sync
  - init with file backend
  - equip install fixture --agent claude --local
  - verify NO op file created (local installs don't sync)

auto_sync_failure_does_not_fail_install
  - init with file backend pointing to non-writable dir
  - equip install fixture --agent claude
  - verify install succeeds (skill present)
  - verify warning about sync failure printed
```

### Integration tests: `tests/cli.rs` — Ops roundtrip

```
full_roundtrip_file_backend
  - init --path {tempdir}
  - install 2 skills globally
  - equip export (writes ops)
  - remove both skills
  - equip restore (reinstalls from ops)
  - verify both skills present again

export_then_restore_via_file
  - install skills, equip export --output {file}
  - remove all skills
  - equip restore --from {file}
  - verify skills restored
```

## Verification

1. `cargo build && cargo test` — all tests pass
2. `cargo clippy` — no warnings
3. Manual smoke test with real GitHub repo (optional)
