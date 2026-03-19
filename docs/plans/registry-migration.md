Status: Completed

# Registry Migration: .equip.json -> ~/.equip/registry.json

## Summary

Replaced all `.equip.json` sidecar file usage with a centralized registry at `~/.equip/registry.json`.

## Design

File: `~/.equip/registry.json`
```json
{
  "version": 1,
  "entries": {
    "global/agg": { ... },
    "/Users/brad/dev/myproject/agg": { ... }
  }
}
```

Key = `<scope>/<skill_name>` where scope is "global" or absolute project path.

## Changes Made

### New file: `src/registry.rs`
- `Registry` struct with `load()`, `save()` (atomic), `upsert()` (merges agents), `remove_entry()`, `remove_agents()`, `get()`, `entries_for_scope()`
- `RegistryEntry` struct with `as_metadata()` conversion method
- Helper functions: `scope_global()`, `scope_for_project()`, `find_skill_path()`
- Unit tests for all operations

### Modified files
- `src/lib.rs` / `src/main.rs` - Added `registry` module
- `src/metadata.rs` - Removed `read()` and `write()` methods, kept struct + date utilities
- `src/commands/install.rs` - Registry upsert after agent loop, delete legacy .equip.json
- `src/commands/remove.rs` - Registry remove_entry/remove_agents after disk removal
- `src/commands/list.rs` - Registry lookup instead of SkillMetadata::read
- `src/commands/update.rs` - Registry scan instead of per-agent SkillMetadata::read
- `src/commands/outdated.rs` - Registry scan instead of per-agent SkillMetadata::read
- `src/commands/status.rs` - Registry lookup instead of SkillMetadata::read
- `src/commands/export.rs` - Registry lookup instead of SkillMetadata::read
- `src/commands/survey.rs` - Registry lookup, updated "unmanaged" message
- `src/commands/fix.rs` - Adopt action writes to registry, updated description string
- `src/hash.rs` - Updated comment on .equip.json exclusion
- `tests/cli.rs` - Updated assertions: registry.json instead of .equip.json

## Cleanup notes
- `.equip.json` files are deleted when encountered (in list, install, survey)
- The `.equip.json` skip in `copy_dir_recursive` and `hash::collect_files` is kept as legacy handling
- `SkillMetadata` struct is preserved in metadata.rs for use by update/outdated (via `as_metadata()`)
