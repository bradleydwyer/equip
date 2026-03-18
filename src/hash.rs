use std::path::Path;

/// FNV-1a hash of a byte slice.
pub fn fnv1a(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

/// Hash all files in a skill directory (excluding .equip.json and .git).
/// Files are sorted by relative path for deterministic output.
/// Returns 0 if the directory cannot be read.
pub fn hash_skill_dir(skill_dir: &Path) -> u64 {
    let mut files = Vec::new();
    if collect_files(skill_dir, skill_dir, &mut files).is_err() {
        return 0;
    }
    files.sort();

    let mut hash: u64 = 0xcbf29ce484222325;
    for (rel_path, contents) in &files {
        // Hash the relative path then file contents into one running hash
        for &byte in rel_path.as_bytes() {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
        for &byte in contents {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(0x100000001b3);
        }
    }
    hash
}

/// Hash only SKILL.md content (used by survey/fix for cross-agent comparison).
pub fn hash_skill_md(skill_dir: &Path) -> u64 {
    let skill_md = skill_dir.join("SKILL.md");
    match std::fs::read(&skill_md) {
        Ok(bytes) => fnv1a(&bytes),
        Err(_) => 0,
    }
}

fn collect_files(base: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) -> Result<(), String> {
    let entries =
        std::fs::read_dir(dir).map_err(|e| format!("Failed to read {}: {e}", dir.display()))?;

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str == ".git" || name_str == ".equip.json" {
            continue;
        }

        if path.is_dir() {
            collect_files(base, &path, out)?;
        } else if let Ok(contents) = std::fs::read(&path) {
            let rel = path
                .strip_prefix(base)
                .unwrap_or(&path)
                .to_string_lossy()
                .to_string();
            out.push((rel, contents));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_known_value() {
        // FNV-1a of empty input is the offset basis
        assert_eq!(fnv1a(&[]), 0xcbf29ce484222325);
    }

    #[test]
    fn fnv1a_deterministic() {
        let data = b"hello world";
        assert_eq!(fnv1a(data), fnv1a(data));
    }

    #[test]
    fn hash_skill_dir_nonexistent_returns_zero() {
        assert_eq!(hash_skill_dir(Path::new("/nonexistent/path")), 0);
    }

    #[test]
    fn hash_skill_dir_deterministic() {
        let dir = std::env::temp_dir().join("equip-hash-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("SKILL.md"), "---\nname: test\n---\n").unwrap();
        std::fs::write(dir.join("README.md"), "hello").unwrap();

        let h1 = hash_skill_dir(&dir);
        let h2 = hash_skill_dir(&dir);
        assert_eq!(h1, h2);
        assert_ne!(h1, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_skill_dir_excludes_equip_json() {
        let dir = std::env::temp_dir().join("equip-hash-exclude-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("SKILL.md"), "---\nname: test\n---\n").unwrap();

        let h1 = hash_skill_dir(&dir);

        // Adding .equip.json should not change the hash
        std::fs::write(dir.join(".equip.json"), "{}").unwrap();
        let h2 = hash_skill_dir(&dir);
        assert_eq!(h1, h2);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn hash_skill_md_reads_skill_file() {
        let dir = std::env::temp_dir().join("equip-hash-md-test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("SKILL.md"), "test content").unwrap();

        let h = hash_skill_md(&dir);
        assert_ne!(h, 0);
        assert_eq!(h, fnv1a(b"test content"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
