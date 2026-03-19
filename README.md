# equip

<p align="center">
  <img src="logos/equip-banner.png" alt="equip — RPG armory banner" />
</p>

Install and sync SKILL.md files across AI coding agents and machines. One command, all your agents.

Single static binary. No Node.js, no Python. Just install and go.

<p align="center">
  <img src="demos/equip-init.gif" alt="equip demo — init, install, and list" />
</p>

> **Warning:** equip is under active development. Expect breaking changes between versions — there are no backwards compatibility guarantees yet.

## First Setup

```bash
brew install bradleydwyer/tap/equip
```

Or download a binary from [Releases](https://github.com/bradleydwyer/equip/releases).

equip auto-detects which agents are installed: Claude Code, Codex, Gemini CLI, OpenCode, pi-mono, Amp, Cline, Continue, Cursor, GitHub Copilot, Goose, Kilo Code, Kiro, Pear AI, Roo Code, Sourcegraph Cody, Windsurf, Zed.

## Add a Skill

```bash
equip install anthropics/skills/skills/pdf  # from GitHub
equip install ./my-skill                   # from a local path
equip install ./my-skill --local           # project-local scope
equip install ./my-skill --agent claude    # specific agent(s)
```

## Remove a Skill

```bash
equip remove my-skill
```

## Check for Updates

```bash
equip outdated                             # see what's changed
equip update                               # re-install from source
```

`equip outdated` detects upstream changes and local modifications. `equip update` updates skills that have changed (skips up-to-date, warns about local edits).

## Adopt Existing Skills

Already have skills in `~/.claude/skills/` that weren't installed with equip?

```bash
equip survey                               # find unmanaged skills
equip survey --fix                         # adopt them interactively
```

## Cross-Machine Sync

```bash
equip init                                 # link to GitHub repo (defaults to <gh-user>/loadout)
equip init --path ~/iCloud/equip/          # or use a file path

# install/remove auto-sync after init
equip install anthropics/skills/skills/pdf  # synced automatically

# on a new machine
equip init
equip restore                              # install everything from the manifest
equip status                               # check sync state
```

Reference skills from other repos with an `includes` file in your sync repo:

```
bradleydwyer/available
bradleydwyer/sloppy/skill
anthropics/skills/skills/pdf
```

## More

```bash
equip list                                 # list installed skills
equip survey --path ~/dev                  # scan all projects for skill issues
equip config projects_path ~/dev           # set default survey path
equip agents                               # generate AGENTS.md
```

Every command supports `--json` and most support `--local` for project scope.

## License

MIT
