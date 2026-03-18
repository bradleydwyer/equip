use std::process::Command;

use crate::config::{self, EquipConfig};
use crate::ops;
use crate::output;
use crate::source::SkillSource;

pub fn run(source: Option<&str>, path: Option<&str>, protocol: Option<&str>) -> Result<(), String> {
    if source.is_some() && path.is_some() {
        return Err("Provide either a GitHub repo or --path, not both.".to_string());
    }

    if let Some(p) = protocol
        && p != "ssh"
        && p != "https"
    {
        return Err("--protocol must be 'ssh' or 'https'".to_string());
    }

    if let Some(path_str) = path {
        return init_file_backend(path_str);
    }

    if let Some(source_str) = source {
        return init_git_backend(source_str, protocol);
    }

    // No source or path — default to <gh-user>/loadout
    let default_source = resolve_default_repo()?;
    init_git_backend(&default_source, protocol)
}

fn resolve_default_repo() -> Result<String, String> {
    let output = Command::new("gh")
        .args(["api", "user", "--jq", ".login"])
        .output()
        .map_err(|e| format!("Failed to run gh: {e}"))?;

    if !output.status.success() {
        return Err(
            "Could not detect GitHub user. Either log in with `gh auth login` or specify a repo: equip init user/repo"
                .to_string(),
        );
    }

    let username = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if username.is_empty() {
        return Err(
            "Could not detect GitHub user. Either log in with `gh auth login` or specify a repo: equip init user/repo"
                .to_string(),
        );
    }

    let repo = format!("{username}/loadout");
    println!("Using default repo: {}", repo);
    Ok(repo)
}

fn init_file_backend(path_str: &str) -> Result<(), String> {
    let path = std::path::PathBuf::from(path_str);

    // Create directory and ops subdirectory
    let ops_path = path.join("ops");
    std::fs::create_dir_all(&ops_path)
        .map_err(|e| format!("Failed to create {}: {e}", ops_path.display()))?;

    let config = EquipConfig::File {
        path: path.display().to_string(),
    };
    config::write(&config)?;

    // Check for existing ops
    let state = ops::compute_state(&ops_path)?;
    if !state.is_empty() {
        println!(
            "{} Found {} skill(s) in sync log. Run {} to install them.",
            output::green("✓"),
            state.len(),
            output::bold("equip restore")
        );
    } else {
        println!(
            "{} Linked equip to file path: {}",
            output::green("✓"),
            path.display()
        );
    }

    Ok(())
}

fn init_git_backend(source_str: &str, protocol: Option<&str>) -> Result<(), String> {
    let source = SkillSource::parse(source_str)?;
    let repo_shorthand = match &source {
        SkillSource::GitHub { owner, repo, .. } => format!("{owner}/{repo}"),
        SkillSource::GitUrl { .. } => {
            return Err(
                "Use owner/repo shorthand for init, not a full URL (e.g., equip init user/repo)"
                    .to_string(),
            );
        }
        SkillSource::Local { .. } => {
            return Err(
                "Use --path for local directory sync, not a local path as source.".to_string(),
            );
        }
    };

    let gh_available = Command::new("gh").arg("--version").output().is_ok();
    if !gh_available {
        return Err("gh CLI is required for git sync. Install it: brew install gh".to_string());
    }

    let repo_dir = config::repo_dir()?;
    let equip_dir = config::equip_dir()?;
    std::fs::create_dir_all(&equip_dir)
        .map_err(|e| format!("Failed to create {}: {e}", equip_dir.display()))?;

    // Clone to a temp dir first so we don't destroy the existing repo on failure
    let temp_repo = equip_dir.join("repo.tmp");
    if temp_repo.exists() {
        let _ = std::fs::remove_dir_all(&temp_repo);
    }

    // Clone the repo, with protocol selection and auto-fallback
    let clone_ok = clone_with_fallback(&repo_shorthand, &temp_repo, protocol)?;

    if !clone_ok {
        // Repo doesn't exist — create it, then clone
        println!("Creating repo {}...", &repo_shorthand);
        let create = Command::new("gh")
            .args(["repo", "create", &repo_shorthand, "--public"])
            .output()
            .map_err(|e| format!("Failed to run gh repo create: {e}"))?;

        if !create.status.success() {
            let _ = std::fs::remove_dir_all(&temp_repo);
            let create_stderr = String::from_utf8_lossy(&create.stderr);
            return Err(format!("Failed to create repo: {}", create_stderr.trim()));
        }

        // Clone the newly created repo
        let clone_ok2 = clone_with_fallback(&repo_shorthand, &temp_repo, protocol)?;
        if !clone_ok2 {
            let _ = std::fs::remove_dir_all(&temp_repo);
            return Err("Failed to clone newly created repo".to_string());
        }

        // Create ops directory, README, and initial commit
        let ops_path = temp_repo.join("ops");
        std::fs::create_dir_all(&ops_path).map_err(|e| format!("Failed to create ops dir: {e}"))?;

        std::fs::write(ops_path.join(".gitkeep"), "")
            .map_err(|e| format!("Failed to write .gitkeep: {e}"))?;

        std::fs::write(temp_repo.join("README.md"), loadout_readme(&repo_shorthand))
            .map_err(|e| format!("Failed to write README.md: {e}"))?;

        let repo_str = temp_repo.display().to_string();
        run_git(&repo_str, &["add", "."])?;
        run_git(&repo_str, &["commit", "-m", "init equip sync"])?;
        run_git(&repo_str, &["push"])?;
    } else {
        // Ensure ops directory exists in the cloned repo
        let ops_path = temp_repo.join("ops");
        if !ops_path.exists() {
            std::fs::create_dir_all(&ops_path)
                .map_err(|e| format!("Failed to create ops dir: {e}"))?;
        }
    }

    // Set a default git identity in the repo so commits work on machines
    // without a global git config
    let temp_repo_str = temp_repo.display().to_string();
    let _ = run_git(&temp_repo_str, &["config", "user.name", "equip"]);
    let _ = run_git(&temp_repo_str, &["config", "user.email", "equip@local"]);

    // Clone succeeded — swap temp into place
    if repo_dir.exists() {
        std::fs::remove_dir_all(&repo_dir)
            .map_err(|e| format!("Failed to clean up old repo: {e}"))?;
    }
    std::fs::rename(&temp_repo, &repo_dir)
        .map_err(|e| format!("Failed to move repo into place: {e}"))?;

    // Store the repo URL from the cloned repo (uses whatever protocol gh chose)
    let repo_url = get_remote_url(&repo_dir).unwrap_or_else(|| repo_shorthand.clone());

    let config = EquipConfig::Git {
        repo: repo_shorthand.clone(),
        repo_url,
    };
    config::write(&config)?;

    // Check for existing ops
    let ops_path = repo_dir.join("ops");
    let state = ops::compute_state(&ops_path)?;
    if !state.is_empty() {
        println!(
            "{} Linked to {}. Found {} skill(s) in sync log. Run {} to install them.",
            output::green("✓"),
            &repo_shorthand,
            state.len(),
            output::bold("equip restore")
        );
    } else {
        println!("{} Linked equip to {}", output::green("✓"), &repo_shorthand);
    }

    Ok(())
}

/// Clone a repo with protocol selection and auto-fallback.
/// Returns Ok(true) if clone succeeded, Ok(false) if repo not found, Err on hard failure.
fn clone_with_fallback(
    repo_shorthand: &str,
    dest: &std::path::Path,
    protocol: Option<&str>,
) -> Result<bool, String> {
    let ssh_url = format!("git@github.com:{repo_shorthand}.git");
    let https_url = format!("https://github.com/{repo_shorthand}.git");

    // Build list of URLs to try
    let urls: Vec<(&str, &str)> = match protocol {
        Some("ssh") => vec![("ssh", &ssh_url)],
        Some("https") => vec![("https", &https_url)],
        _ => {
            // Auto-detect: try gh's preferred protocol first, fall back to the other
            let gh_protocol = Command::new("gh")
                .args(["config", "get", "git_protocol"])
                .output()
                .ok()
                .and_then(|o| {
                    if o.status.success() {
                        Some(String::from_utf8_lossy(&o.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "https".to_string());

            if gh_protocol == "ssh" {
                vec![("ssh", &ssh_url), ("https", &https_url)]
            } else {
                vec![("https", &https_url), ("ssh", &ssh_url)]
            }
        }
    };

    for (proto_name, url) in &urls {
        // Clean up any partial clone from a previous attempt
        if dest.exists() {
            let _ = std::fs::remove_dir_all(dest);
        }

        let result = Command::new("git")
            .args(["clone", "--quiet", url])
            .arg(dest)
            .output()
            .map_err(|e| format!("Failed to run git clone: {e}"))?;

        if result.status.success() {
            return Ok(true);
        }

        let stderr = String::from_utf8_lossy(&result.stderr);

        // Repo doesn't exist — no point trying another protocol
        if stderr.contains("not found")
            || stderr.contains("does not exist")
            || stderr.contains("Repository not found")
        {
            return Ok(false);
        }

        // Connection/auth error — try next protocol
        if (stderr.contains("timed out")
            || stderr.contains("Connection refused")
            || stderr.contains("Host key verification failed")
            || stderr.contains("Permission denied")
            || stderr.contains("Could not read from remote repository"))
            && urls.len() > 1
        {
            eprintln!(
                "Warning: {} clone failed, trying {}...",
                proto_name,
                if *proto_name == "ssh" { "https" } else { "ssh" }
            );
            continue;
        }

        // Unknown error
        let _ = std::fs::remove_dir_all(dest);
        return Err(format!(
            "git clone failed ({}): {}",
            proto_name,
            stderr.trim()
        ));
    }

    // All protocols failed
    let _ = std::fs::remove_dir_all(dest);
    Err("Failed to clone repo via both SSH and HTTPS".to_string())
}

fn get_remote_url(repo_dir: &std::path::Path) -> Option<String> {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_dir.display().to_string(),
            "remote",
            "get-url",
            "origin",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn loadout_readme(repo_shorthand: &str) -> String {
    format!(
        r#"# {repo_shorthand}

My skills for AI coding agents, managed with [equip](https://github.com/bradleydwyer/equip).

## Setup

```bash
brew install bradleydwyer/tap/equip
equip init {repo_shorthand}
equip restore
```

## What is equip?

[equip](https://github.com/bradleydwyer/equip) installs SKILL.md files across every AI coding agent on your machine — Claude Code, Cursor, Codex, Gemini CLI, and [14 more](https://github.com/bradleydwyer/equip#first-setup). One command, all your agents.

```bash
equip install owner/repo          # add a skill from GitHub
equip outdated                    # check for updates
equip update                      # update all skills
```
"#
    )
}

fn run_git(repo_dir: &str, args: &[&str]) -> Result<(), String> {
    let mut full_args = vec!["-C", repo_dir];
    full_args.extend_from_slice(args);
    let output = Command::new("git")
        .args(&full_args)
        .output()
        .map_err(|e| format!("Failed to run git {}: {e}", args.first().unwrap_or(&"")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "git {} failed: {}",
            args.first().unwrap_or(&""),
            stderr.trim()
        ));
    }
    Ok(())
}
