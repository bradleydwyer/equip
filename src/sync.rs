use std::process::Command;

use crate::config::{self, EquipConfig};
use crate::ops::{self, Op};

/// Pull latest from backend (git pull for Git, no-op for File)
pub fn pull(config: &EquipConfig) -> Result<(), String> {
    match config {
        EquipConfig::Git { .. } => {
            let repo = config::repo_dir()?;
            if !repo.exists() {
                return Err("Sync repo not found. Run 'equip init' first.".to_string());
            }
            let repo_str = repo.display().to_string();

            // Reset any dirty state from a previously failed sync
            let _ = Command::new("git")
                .args(["-C", &repo_str, "reset", "--hard", "HEAD"])
                .output();
            let _ = Command::new("git")
                .args(["-C", &repo_str, "clean", "-fd"])
                .output();

            let output = Command::new("git")
                .args(["-C", &repo_str, "pull", "--rebase", "--quiet"])
                .output()
                .map_err(|e| format!("Failed to run git pull: {e}"))?;
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("git pull failed: {}", stderr.trim()));
            }
            Ok(())
        }
        EquipConfig::File { .. } => Ok(()), // iCloud/Dropbox syncs automatically
    }
}

/// Push new ops to backend (git add + commit + push for Git, no-op for File)
pub fn push(config: &EquipConfig) -> Result<(), String> {
    match config {
        EquipConfig::Git { .. } => {
            let repo = config::repo_dir()?;
            let repo_str = repo.display().to_string();

            // Check if there are changes to commit (unstaged, staged, or untracked)
            let diff_unstaged = Command::new("git")
                .args(["-C", &repo_str, "diff", "--quiet", "--exit-code"])
                .status()
                .map_err(|e| format!("Failed to run git diff: {e}"))?;

            let diff_staged = Command::new("git")
                .args([
                    "-C",
                    &repo_str,
                    "diff",
                    "--cached",
                    "--quiet",
                    "--exit-code",
                ])
                .status()
                .map_err(|e| format!("Failed to run git diff --cached: {e}"))?;

            let untracked = Command::new("git")
                .args([
                    "-C",
                    &repo_str,
                    "ls-files",
                    "--others",
                    "--exclude-standard",
                ])
                .output()
                .map_err(|e| format!("Failed to check untracked files: {e}"))?;

            let has_untracked = !String::from_utf8_lossy(&untracked.stdout).trim().is_empty();

            if diff_unstaged.success() && diff_staged.success() && !has_untracked {
                // No changes
                return Ok(());
            }

            // Update loadout README
            if let EquipConfig::Git {
                repo: shorthand, ..
            } = config
            {
                let _ = std::fs::write(
                    repo.join("README.md"),
                    crate::commands::init::loadout_readme(shorthand),
                );
            }

            // Stage ops/, skills/, and README (only paths that exist)
            let mut add_args = vec!["-C", &repo_str, "add"];
            let ops_path = repo.join(".ops");
            let skills_path = repo.join("skills");
            let readme_path = repo.join("README.md");
            let ops_str = ops_path.display().to_string();
            let skills_str = skills_path.display().to_string();
            let readme_str = readme_path.display().to_string();
            if ops_path.exists() {
                add_args.push(&ops_str);
            }
            if skills_path.exists() {
                add_args.push(&skills_str);
            }
            if readme_path.exists() {
                add_args.push(&readme_str);
            }
            if add_args.len() == 3 {
                // Nothing to stage
                return Ok(());
            }
            let add = Command::new("git")
                .args(&add_args)
                .output()
                .map_err(|e| format!("Failed to run git add: {e}"))?;
            if !add.status.success() {
                return Err(format!(
                    "git add failed: {}",
                    String::from_utf8_lossy(&add.stderr).trim()
                ));
            }

            // Commit
            let commit = Command::new("git")
                .args(["-C", &repo_str, "commit", "-m", "equip sync"])
                .output()
                .map_err(|e| format!("Failed to run git commit: {e}"))?;
            if !commit.status.success() {
                let stderr = String::from_utf8_lossy(&commit.stderr);
                // "nothing to commit" is not an error
                if !stderr.contains("nothing to commit") {
                    return Err(format!("git commit failed: {}", stderr.trim()));
                }
                return Ok(());
            }

            // Push
            let push_out = Command::new("git")
                .args(["-C", &repo_str, "push", "--quiet"])
                .output()
                .map_err(|e| format!("Failed to run git push: {e}"))?;
            if !push_out.status.success() {
                return Err(format!(
                    "git push failed: {}",
                    String::from_utf8_lossy(&push_out.stderr).trim()
                ));
            }

            Ok(())
        }
        EquipConfig::File { .. } => Ok(()), // File backend: sync happens via iCloud/Dropbox
    }
}

/// Write an op and push to the backend. Best-effort: returns Ok even if push fails (with warning printed).
pub fn write_and_push(config: &EquipConfig, op: &Op) -> Result<(), String> {
    let ops_dir = config::ops_dir(config)?;
    ops::write_op(&ops_dir, op)?;

    if let Err(e) = push(config) {
        eprintln!("Warning: sync failed: {e}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify that the module compiles and the public API is accessible.
    #[test]
    fn module_compiles() {
        // The existence of pull, push, and write_and_push is confirmed at compile time.
        // We reference them here to ensure they remain public.
        let _ = pull as fn(&EquipConfig) -> Result<(), String>;
        let _ = push as fn(&EquipConfig) -> Result<(), String>;
        let _ = write_and_push as fn(&EquipConfig, &Op) -> Result<(), String>;
    }
}
