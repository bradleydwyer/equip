use clap::{Parser, Subcommand};
use std::process;

mod agents;
mod commands;
mod config;
mod hash;
mod metadata;
mod ops;
mod output;
mod skill;
mod source;
mod sync;

#[derive(Parser)]
#[command(
    name = "equip",
    version,
    about = "Install SKILL.md files across AI coding agents"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Install skills from a GitHub repo, git URL, or local path
    Install {
        /// Source: owner/repo, git URL, or local path
        source: String,

        /// Install to project-local scope (default: global)
        #[arg(short, long)]
        local: bool,

        /// Target specific agent(s) by id
        #[arg(short, long, value_delimiter = ',')]
        agent: Vec<String>,

        /// Install for all known agents regardless of detection
        #[arg(long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Remove an installed skill
    #[command(alias = "uninstall")]
    Remove {
        /// Skill name to remove
        name: String,

        /// Remove from project-local scope (default: global)
        #[arg(short, long)]
        local: bool,

        /// Target specific agent(s) by id
        #[arg(short, long, value_delimiter = ',')]
        agent: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List installed skills
    List {
        /// List project-local skills (default: global)
        #[arg(short, long)]
        local: bool,

        /// Show full descriptions
        #[arg(long)]
        long: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Update installed skills from their original source
    Update {
        /// Specific skill to update (default: all)
        name: Option<String>,

        /// Update project-local skills (default: global)
        #[arg(short, long)]
        local: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Survey installed skills for sprawl, mismatches, and issues
    Survey {
        /// Survey project-local skills only (default: global)
        #[arg(short, long)]
        local: bool,

        /// Scan a directory tree for skills across projects
        #[arg(short, long)]
        path: Option<String>,

        /// Interactively fix issues found
        #[arg(long)]
        fix: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate AGENTS.md with installed skills
    #[command(alias = "sync")]
    Agents {
        /// Output file path (default: AGENTS.md)
        #[arg(short, long)]
        output: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Link equip to a sync backend (GitHub repo or file path). Defaults to <gh-user>/equip-loadout
    Init {
        /// GitHub repo (owner/repo). Defaults to <gh-user>/equip-loadout if omitted
        source: Option<String>,

        /// Use a file path as sync backend (iCloud, Dropbox, etc.)
        #[arg(long)]
        path: Option<String>,

        /// Git protocol to use: ssh or https (auto-detects by default, falls back on failure)
        #[arg(long)]
        protocol: Option<String>,

        /// Discard unpushed changes in existing sync repo
        #[arg(long)]
        force: bool,
    },

    /// Export installed skills to sync backend or file
    Export {
        /// Write to file instead of sync backend
        #[arg(short, long)]
        output: Option<String>,

        /// Print as JSON to stdout
        #[arg(long)]
        json: bool,
    },

    /// Restore skills from sync backend or file
    Restore {
        /// Read from file instead of sync backend
        #[arg(long)]
        from: Option<String>,

        /// Show what would be installed without installing
        #[arg(long)]
        dry_run: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show sync state between local installation and manifest
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Check installed skills for upstream or local changes
    Outdated {
        /// Specific skill to check (default: all)
        name: Option<String>,

        /// Check project-local skills (default: global)
        #[arg(short, long)]
        local: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// View or set equip configuration
    Config {
        /// Setting key (e.g., projects_path)
        key: Option<String>,

        /// Value to set (omit to read current value)
        value: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Install {
            source,
            local,
            agent,
            all,
            json,
        } => commands::install::run(&source, !local, &agent, all, json),

        Commands::Remove {
            name,
            local,
            agent,
            json,
        } => commands::remove::run(&name, !local, &agent, json),

        Commands::List { local, long, json } => commands::list::run(!local, json, long),

        Commands::Update { name, local, json } => {
            commands::update::run(name.as_deref(), !local, json)
        }

        Commands::Survey {
            local,
            json,
            path,
            fix,
        } => commands::survey::run(!local, json, path.as_deref(), fix),

        Commands::Agents { output, json } => commands::sync::run(output.as_deref(), json),

        Commands::Init {
            source,
            path,
            protocol,
            force,
        } => commands::init::run(
            source.as_deref(),
            path.as_deref(),
            protocol.as_deref(),
            force,
        ),

        Commands::Export { output, json } => commands::export::run(output.as_deref(), json),

        Commands::Restore {
            from,
            dry_run,
            json,
        } => commands::restore::run(from.as_deref(), dry_run, json),

        Commands::Status { json } => commands::status::run(json),

        Commands::Outdated { name, local, json } => {
            commands::outdated::run(name.as_deref(), !local, json)
        }

        Commands::Config { key, value } => {
            commands::config_cmd::run(key.as_deref(), value.as_deref())
        }
    };

    if let Err(msg) = result {
        eprintln!("{}", output::red(&format!("Error: {msg}")));
        process::exit(1);
    }
}
