// Re-export from equip-lib (except discover_skills which we override with a recursive version)
pub use equip_lib::skill::{SkillFrontmatter, read_includes, read_skill};

use std::path::{Path, PathBuf};

/// Recursively scan a directory for skills, up to a depth limit.
/// This handles repos that nest skills multiple levels deep
/// (e.g. `skills/find-skills/SKILL.md`).
pub fn discover_skills(dir: &Path) -> Result<Vec<(PathBuf, SkillFrontmatter)>, String> {
    let mut skills = Vec::new();
    discover_recursive(dir, 0, 4, &mut skills);

    if skills.is_empty() {
        return Err(format!("No SKILL.md files found in {}", dir.display()));
    }

    skills.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    Ok(skills)
}

fn discover_recursive(
    dir: &Path,
    depth: usize,
    max_depth: usize,
    skills: &mut Vec<(PathBuf, SkillFrontmatter)>,
) {
    if depth > max_depth {
        return;
    }

    if dir.join("SKILL.md").exists() {
        match read_skill(dir) {
            Ok(fm) => skills.push((dir.to_path_buf(), fm)),
            Err(e) => eprintln!("Warning: skipping {}: {e}", dir.display()),
        }
        return; // Don't recurse into skill directories
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') || name_str == "node_modules" || name_str == "target" {
                continue;
            }
            discover_recursive(&path, depth + 1, max_depth, skills);
        }
    }
}
