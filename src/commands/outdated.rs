use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::hash;
use crate::metadata::{self, SkillMetadata};
use crate::output;
use crate::registry;

pub fn run(name: Option<&str>, global: bool, json: bool) -> Result<(), String> {
    let project_root =
        std::env::current_dir().map_err(|e| format!("Failed to get current directory: {e}"))?;

    let reg = registry::Registry::load()?;
    let scope = if global {
        registry::scope_global().to_string()
    } else {
        registry::scope_for_project(&project_root)
    };
    let reg_entries = reg.entries_for_scope(&scope);

    // Collect installed skills with metadata (deduplicated by name)
    let mut skills: Vec<(String, std::path::PathBuf, SkillMetadata)> = Vec::new();

    for entry in reg_entries {
        if let Some(target) = name {
            if entry.skill_name != target {
                continue;
            }
        }

        if let Some(path) = registry::find_skill_path(&entry.skill_name, global, &project_root) {
            skills.push((entry.skill_name.clone(), path, entry.as_metadata()));
        }
    }

    if skills.is_empty() {
        if let Some(target) = name {
            return Err(format!(
                "Skill '{}' not found or has no metadata. Run 'equip list' to see installed skills.",
                target
            ));
        }
        if json {
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "action": "outdated",
                    "global": global,
                    "skills": [],
                }))
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
            );
        } else {
            println!("No skills with metadata found.");
        }
        return Ok(());
    }

    if !json {
        println!("Checking {} skill(s)...\n", skills.len());
    }

    // Batch git ls-remote calls by repo URL
    let remote_info = fetch_remote_info(&skills);

    let mut results: Vec<serde_json::Value> = Vec::new();
    let mut counts = StatusCounts::default();

    for (skill_name, skill_path, meta) in &skills {
        let drift = check_drift(skill_path, meta, &remote_info);

        match &drift.status {
            DriftStatus::UpToDate => counts.up_to_date += 1,
            DriftStatus::UpstreamChanged => counts.upstream += 1,
            DriftStatus::LocallyModified => counts.local += 1,
            DriftStatus::Both => {
                counts.upstream += 1;
                counts.local += 1;
            }
            DriftStatus::Unknown => counts.unknown += 1,
            DriftStatus::LocalSource => counts.up_to_date += 1,
            DriftStatus::Adopted => counts.adopted += 1,
            DriftStatus::CheckFailed(_) => counts.failed += 1,
        }

        if json {
            results.push(drift_to_json(skill_name, meta, &drift));
        } else {
            print_drift(skill_name, &drift);
        }
    }

    if json {
        let out = serde_json::json!({
            "action": "outdated",
            "global": global,
            "skills": results,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out)
                .map_err(|e| format!("Failed to serialize JSON: {e}"))?
        );
    } else {
        println!();
        print_summary(&counts);
    }

    Ok(())
}

// --- Types ---

#[derive(Debug)]
enum DriftStatus {
    UpToDate,
    UpstreamChanged,
    LocallyModified,
    Both,
    Unknown,
    LocalSource,
    Adopted,
    CheckFailed(String),
}

#[derive(Debug, Clone)]
enum VersionLabel {
    Version(String),
    Tag(String),
    Date(String),
    Sha(String),
    Unknown,
}

impl VersionLabel {
    fn display(&self) -> String {
        match self {
            VersionLabel::Version(v) => v.clone(),
            VersionLabel::Tag(t) => t.clone(),
            VersionLabel::Date(d) => d.clone(),
            VersionLabel::Sha(s) => s[..7.min(s.len())].to_string(),
            VersionLabel::Unknown => "?".to_string(),
        }
    }
}

struct DriftResult {
    status: DriftStatus,
    installed_commit: Option<String>,
    remote_commit: Option<String>,
    installed_hash: Option<String>,
    current_hash: Option<String>,
    installed_label: VersionLabel,
    remote_label: VersionLabel,
}

struct RemoteInfo {
    head_sha: String,
    /// Map from commit SHA to tag names
    tags: HashMap<String, Vec<String>>,
}

#[derive(Default)]
struct StatusCounts {
    up_to_date: usize,
    upstream: usize,
    local: usize,
    unknown: usize,
    local_source: usize,
    adopted: usize,
    failed: usize,
}

// --- Version label resolution ---

fn resolve_installed_label(meta: &SkillMetadata) -> VersionLabel {
    if let Some(v) = &meta.version {
        return VersionLabel::Version(v.clone());
    }
    if let Some(tag) = &meta.source_tag {
        return VersionLabel::Tag(tag.clone());
    }
    if let Some(d) = &meta.commit_date {
        return VersionLabel::Date(d.clone());
    }
    if let Some(d) = &meta.source_date {
        return VersionLabel::Date(d.clone());
    }
    if let Some(sha) = &meta.source_commit {
        return VersionLabel::Sha(sha.clone());
    }
    VersionLabel::Unknown
}

fn resolve_remote_label(remote: &RemoteInfo) -> VersionLabel {
    if let Some(tags) = remote.tags.get(&remote.head_sha)
        && let Some(best) = pick_best_tag(tags)
    {
        return VersionLabel::Tag(best);
    }
    VersionLabel::Sha(remote.head_sha.clone())
}

fn resolve_local_remote_label(meta: &SkillMetadata) -> VersionLabel {
    if let Some(local_path) = &meta.local_path {
        let source_path = Path::new(local_path);
        // Try SKILL.md version field
        if let Ok(fm) = crate::skill::read_skill(source_path)
            && let Some(v) = fm.version
        {
            return VersionLabel::Version(v);
        }
        // Fallback: SKILL.md mtime
        if let Ok(m) = std::fs::metadata(source_path.join("SKILL.md"))
            && let Ok(mtime) = m.modified()
            && let Some(date) = metadata::system_time_to_date(mtime)
        {
            return VersionLabel::Date(date);
        }
    }
    VersionLabel::Unknown
}

fn pick_best_tag(tags: &[String]) -> Option<String> {
    let mut candidates = tags.to_vec();
    candidates.sort_by(|a, b| {
        let a_ver = looks_like_version(a);
        let b_ver = looks_like_version(b);
        match (a_ver, b_ver) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => b.cmp(a),
        }
    });
    candidates.into_iter().next()
}

fn looks_like_version(s: &str) -> bool {
    let s = s.strip_prefix('v').unwrap_or(s);
    s.starts_with(|c: char| c.is_ascii_digit()) && s.contains('.')
}

// --- Drift checking ---

fn check_drift(
    skill_path: &Path,
    meta: &SkillMetadata,
    remote_info: &HashMap<String, Result<RemoteInfo, String>>,
) -> DriftResult {
    let installed_label = resolve_installed_label(meta);

    // Adopted skills: no real source to compare against
    if meta.source == "adopted" {
        return DriftResult {
            status: DriftStatus::Adopted,
            installed_commit: None,
            remote_commit: None,
            installed_hash: meta.content_hash.clone(),
            current_hash: Some(format!("{:016x}", hash::hash_skill_dir(skill_path))),
            installed_label,
            remote_label: VersionLabel::Unknown,
        };
    }

    // Local sources: compare installed copy to source directory and check for local edits
    if meta.source_type == "local" {
        let (installed_hash, current_hash, locally_modified) = check_local_drift(skill_path, meta);

        let source_changed = match &meta.local_path {
            Some(local_path) => {
                let source_path = Path::new(local_path);
                if source_path.exists() {
                    match &meta.content_hash {
                        Some(installed) => {
                            let source_hash = format!("{:016x}", hash::hash_skill_dir(source_path));
                            &source_hash != installed
                        }
                        None => false,
                    }
                } else {
                    false
                }
            }
            None => false,
        };

        let remote_label = if source_changed {
            resolve_local_remote_label(meta)
        } else {
            VersionLabel::Unknown
        };

        let status = match (source_changed, locally_modified) {
            (true, true) => DriftStatus::Both,
            (true, false) => DriftStatus::UpstreamChanged,
            (false, true) => DriftStatus::LocallyModified,
            (false, false) => DriftStatus::LocalSource,
        };

        return DriftResult {
            status,
            installed_commit: None,
            remote_commit: None,
            installed_hash,
            current_hash,
            installed_label,
            remote_label,
        };
    }

    // No tracking metadata: unknown
    if meta.source_commit.is_none() && meta.content_hash.is_none() {
        return DriftResult {
            status: DriftStatus::Unknown,
            installed_commit: None,
            remote_commit: None,
            installed_hash: None,
            current_hash: None,
            installed_label,
            remote_label: VersionLabel::Unknown,
        };
    }

    let (installed_hash, current_hash, locally_modified) = check_local_drift(skill_path, meta);

    // Check upstream
    let (upstream_changed, remote_commit, remote_label) = match &meta.repo_url {
        Some(url) => match remote_info.get(url) {
            Some(Ok(info)) => {
                let changed = match &meta.source_commit {
                    Some(installed) => installed != &info.head_sha,
                    None => false,
                };
                let label = if changed {
                    resolve_remote_label(info)
                } else {
                    VersionLabel::Unknown
                };
                (changed, Some(info.head_sha.clone()), label)
            }
            Some(Err(e)) => {
                return DriftResult {
                    status: DriftStatus::CheckFailed(e.clone()),
                    installed_commit: meta.source_commit.clone(),
                    remote_commit: None,
                    installed_hash,
                    current_hash,
                    installed_label,
                    remote_label: VersionLabel::Unknown,
                };
            }
            None => (false, None, VersionLabel::Unknown),
        },
        None => (false, None, VersionLabel::Unknown),
    };

    let status = match (upstream_changed, locally_modified) {
        (true, true) => DriftStatus::Both,
        (true, false) => DriftStatus::UpstreamChanged,
        (false, true) => DriftStatus::LocallyModified,
        (false, false) => DriftStatus::UpToDate,
    };

    DriftResult {
        status,
        installed_commit: meta.source_commit.clone(),
        remote_commit,
        installed_hash,
        current_hash,
        installed_label,
        remote_label,
    }
}

fn check_local_drift(
    skill_path: &Path,
    meta: &SkillMetadata,
) -> (Option<String>, Option<String>, bool) {
    let installed_hash = meta.content_hash.clone();
    let current_hash = Some(format!("{:016x}", hash::hash_skill_dir(skill_path)));

    let locally_modified = match (&installed_hash, &current_hash) {
        (Some(installed), Some(current)) => installed != current,
        _ => false,
    };

    (installed_hash, current_hash, locally_modified)
}

// --- Remote fetching ---

fn fetch_remote_info(
    skills: &[(String, std::path::PathBuf, SkillMetadata)],
) -> HashMap<String, Result<RemoteInfo, String>> {
    let mut urls: Vec<String> = skills
        .iter()
        .filter(|(_, _, m)| m.source_type != "local")
        .filter_map(|(_, _, m)| m.repo_url.clone())
        .collect();
    urls.sort();
    urls.dedup();

    let mut results = HashMap::new();
    for url in urls {
        results.insert(url.clone(), fetch_single_remote(&url));
    }
    results
}

fn fetch_single_remote(repo_url: &str) -> Result<RemoteInfo, String> {
    let head_sha = remote_head_sha(repo_url)?;
    let tags = fetch_remote_tags(repo_url).unwrap_or_default();
    Ok(RemoteInfo { head_sha, tags })
}

fn remote_head_sha(repo_url: &str) -> Result<String, String> {
    let output = Command::new("git")
        .args(["ls-remote", "--quiet", repo_url, "HEAD"])
        .output()
        .map_err(|e| format!("Failed to run git ls-remote: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.trim().to_string());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout
        .split_whitespace()
        .next()
        .map(String::from)
        .ok_or_else(|| "empty repository (no commits)".to_string())
}

fn fetch_remote_tags(repo_url: &str) -> Result<HashMap<String, Vec<String>>, String> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", repo_url])
        .output()
        .map_err(|e| format!("Failed to run git ls-remote --tags: {e}"))?;

    if !output.status.success() {
        return Ok(HashMap::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut lightweight: HashMap<String, Vec<String>> = HashMap::new();
    let mut peeled: HashMap<String, String> = HashMap::new(); // tag_name -> peeled commit SHA

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 2 {
            continue;
        }
        let sha = parts[0];
        let refname = parts[1];

        if let Some(tag_ref) = refname.strip_prefix("refs/tags/") {
            if let Some(tag_name) = tag_ref.strip_suffix("^{}") {
                // Peeled (annotated tag) — this SHA is the actual commit
                peeled.insert(tag_name.to_string(), sha.to_string());
            } else {
                // Lightweight tag or annotated tag object
                lightweight
                    .entry(sha.to_string())
                    .or_default()
                    .push(tag_ref.to_string());
            }
        }
    }

    // Build SHA→tags map: use peeled SHA for annotated tags, direct SHA for lightweight
    let mut result: HashMap<String, Vec<String>> = HashMap::new();
    for (sha, tags) in &lightweight {
        for tag in tags {
            let commit_sha = peeled.get(tag).unwrap_or(sha);
            result
                .entry(commit_sha.clone())
                .or_default()
                .push(tag.clone());
        }
    }

    Ok(result)
}

// --- Display ---

fn format_version_change(
    installed: &VersionLabel,
    remote: &VersionLabel,
    drift: &DriftResult,
) -> String {
    let inst = installed.display();
    let rem = remote.display();
    if inst == "?" && rem == "?" {
        return String::new();
    }
    // If both labels are identical (e.g. same date), fall back to SHAs to show the difference
    if inst == rem
        && let (Some(ic), Some(rc)) = (&drift.installed_commit, &drift.remote_commit)
    {
        return format!("  ({}→{})", &ic[..7.min(ic.len())], &rc[..7.min(rc.len())]);
    }
    format!("  ({}→{})", inst, rem)
}

fn print_drift(skill_name: &str, drift: &DriftResult) {
    let name = output::bold(skill_name);
    match &drift.status {
        DriftStatus::UpToDate => {
            let label = drift.installed_label.display();
            let detail = if label != "?" {
                format!(" {}", output::dim(&label))
            } else {
                String::new()
            };
            println!("  {} {}{}", output::green("✓"), name, detail);
        }
        DriftStatus::UpstreamChanged => {
            let detail = format_version_change(&drift.installed_label, &drift.remote_label, drift);
            println!(
                "  {} {} upstream changed{}",
                output::cyan("↑"),
                name,
                output::dim(&detail)
            );
        }
        DriftStatus::LocallyModified => {
            println!("  {} {} locally modified", output::yellow("~"), name);
        }
        DriftStatus::Both => {
            let detail = format_version_change(&drift.installed_label, &drift.remote_label, drift);
            println!(
                "  {} {} upstream changed + locally modified{}",
                output::cyan("↑~"),
                name,
                output::dim(&detail)
            );
        }
        DriftStatus::Unknown => {
            println!(
                "  {} {} {}",
                output::dim("?"),
                name,
                output::dim("unknown (reinstall to enable tracking)")
            );
        }
        DriftStatus::LocalSource => {
            let label = drift.installed_label.display();
            let detail = if label != "?" {
                format!(" {}", output::dim(&label))
            } else {
                String::new()
            };
            println!("  {} {}{}", output::green("✓"), name, detail);
        }
        DriftStatus::Adopted => {
            println!(
                "  {} {} {}",
                output::dim("·"),
                name,
                output::dim("adopted (no source)")
            );
        }
        DriftStatus::CheckFailed(err) => {
            println!(
                "  {} {} {}",
                output::red("✗"),
                name,
                output::dim(&format!("check failed: {err}"))
            );
        }
    }
}

fn print_summary(counts: &StatusCounts) {
    let mut parts = Vec::new();
    if counts.up_to_date > 0 {
        parts.push(format!("{} up to date", counts.up_to_date));
    }
    if counts.upstream > 0 {
        parts.push(format!("{} upstream changed", counts.upstream));
    }
    if counts.local > 0 {
        parts.push(format!("{} locally modified", counts.local));
    }
    if counts.local_source > 0 {
        parts.push(format!("{} local source", counts.local_source));
    }
    if counts.adopted > 0 {
        parts.push(format!("{} adopted", counts.adopted));
    }
    if counts.unknown > 0 {
        parts.push(format!("{} unknown", counts.unknown));
    }
    if counts.failed > 0 {
        parts.push(format!("{} failed", counts.failed));
    }
    if !parts.is_empty() {
        println!("{}", parts.join(", "));
    }
}

fn drift_to_json(skill_name: &str, meta: &SkillMetadata, drift: &DriftResult) -> serde_json::Value {
    let status = match &drift.status {
        DriftStatus::UpToDate => "up_to_date",
        DriftStatus::UpstreamChanged => "upstream_changed",
        DriftStatus::LocallyModified => "locally_modified",
        DriftStatus::Both => "both",
        DriftStatus::Unknown => "unknown",
        DriftStatus::LocalSource => "local_source",
        DriftStatus::Adopted => "adopted",
        DriftStatus::CheckFailed(_) => "check_failed",
    };

    let mut obj = serde_json::json!({
        "name": skill_name,
        "source": meta.source,
        "source_type": meta.source_type,
        "status": status,
        "installed_version": drift.installed_label.display(),
        "remote_version": drift.remote_label.display(),
    });

    if let Some(c) = &drift.installed_commit {
        obj["installed_commit"] = serde_json::Value::String(c.clone());
    }
    if let Some(c) = &drift.remote_commit {
        obj["remote_commit"] = serde_json::Value::String(c.clone());
    }
    if let Some(h) = &drift.installed_hash {
        obj["installed_hash"] = serde_json::Value::String(h.clone());
    }
    if let Some(h) = &drift.current_hash {
        obj["current_hash"] = serde_json::Value::String(h.clone());
    }
    if let DriftStatus::CheckFailed(err) = &drift.status {
        obj["error"] = serde_json::Value::String(err.clone());
    }

    obj
}
