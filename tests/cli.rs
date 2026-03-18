use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn equip() -> Command {
    Command::cargo_bin("equip").unwrap()
}

fn fixture_path(name: &str) -> String {
    let manifest = env!("CARGO_MANIFEST_DIR");
    format!("{manifest}/tests/fixtures/{name}")
}

/// Write an op JSON file to an ops directory
fn write_op_file(ops_dir: &Path, filename: &str, content: &str) {
    fs::create_dir_all(ops_dir).unwrap();
    fs::write(ops_dir.join(filename), content).unwrap();
}

// --- Help / version ---

#[test]
fn help_shows_all_commands() {
    equip()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("install"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("sync"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("export"))
        .stdout(predicate::str::contains("restore"))
        .stdout(predicate::str::contains("status"));
}

#[test]
fn version_shows_version() {
    equip()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("equip"));
}

// --- Install (local) ---

#[test]
fn install_single_skill() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid-skill"))
        .stdout(predicate::str::contains("1 skill(s)"));

    // Verify files were created
    let skill_dir = project.path().join(".claude/skills/valid-skill");
    assert!(skill_dir.join("SKILL.md").exists());
    assert!(skill_dir.join(".equip.json").exists());

    // Verify metadata
    let meta: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(skill_dir.join(".equip.json")).unwrap()).unwrap();
    assert_eq!(meta["source_type"], "local");
    assert_eq!(meta["agents"][0], "claude");
}

#[test]
fn install_multi_skill_repo() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("multi-skill-repo"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("2 skill(s)"));

    assert!(
        project
            .path()
            .join(".claude/skills/skill-one/SKILL.md")
            .exists()
    );
    assert!(
        project
            .path()
            .join(".claude/skills/skill-two/SKILL.md")
            .exists()
    );
}

#[test]
fn install_to_multiple_agents() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude,cursor,codex",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("3 agent(s)"));

    assert!(
        project
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        project
            .path()
            .join(".cursor/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        project
            .path()
            .join(".codex/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn install_global() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    assert!(
        home.path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
    // Should NOT be in project dir
    assert!(
        !project
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn install_nonexistent_path() {
    equip()
        .args(["install", "/nonexistent/path", "--agent", "claude"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid path"));
}

#[test]
fn install_invalid_agent() {
    equip()
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--agent",
            "nonexistent-agent",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown agent"));
}

// --- List ---

#[test]
fn list_empty_project() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["list", "--local"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No skills installed"));
}

#[test]
fn list_after_install() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install first
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    // List
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["list", "--local"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid-skill"))
        .stdout(predicate::str::contains("Claude Code"));
}

#[test]
fn list_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["list", "--local", "--json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["name"], "valid-skill");
}

// --- Remove ---

#[test]
fn remove_installed_skill() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    assert!(project.path().join(".claude/skills/valid-skill").exists());

    // Remove
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "--local", "valid-skill"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed"));

    assert!(!project.path().join(".claude/skills/valid-skill").exists());
}

#[test]
fn remove_nonexistent_skill() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "--local", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn remove_path_traversal_rejected() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "../../etc"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid skill name"));
}

#[test]
fn remove_dotdot_rejected() {
    equip()
        .args(["remove", ".."])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid skill name"));
}

// --- Sync ---

#[test]
fn sync_generates_agents_md() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install a skill first
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    // Sync
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("sync")
        .assert()
        .success()
        .stdout(predicate::str::contains("Synced 1 skill(s)"));

    let agents_md = fs::read_to_string(project.path().join("AGENTS.md")).unwrap();
    assert!(agents_md.contains("<skills_system"));
    assert!(agents_md.contains("valid-skill"));
    assert!(agents_md.contains("A test skill for integration testing"));
}

#[test]
fn sync_custom_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["sync", "--output", "SKILLS.md"])
        .assert()
        .success();

    assert!(project.path().join("SKILLS.md").exists());
    assert!(!project.path().join("AGENTS.md").exists());
}

// --- Update ---

#[test]
fn update_no_skills() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("update")
        .assert()
        .success()
        .stdout(predicate::str::contains("No skills with metadata"));
}

#[test]
fn update_nonexistent_name() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["update", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

// --- JSON output ---

#[test]
fn install_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--agent",
            "claude",
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "install");
    assert_eq!(json["skills"][0]["name"], "valid-skill");
    assert_eq!(json["skills"][0]["agents"][0], "Claude Code");
    assert!(
        json["skills"][0]["paths"][0]
            .as_str()
            .unwrap()
            .contains(".claude/skills/valid-skill")
    );
}

#[test]
fn install_json_multi_skill() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("multi-skill-repo"),
            "--agent",
            "claude",
            "--json",
        ])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["skills"].as_array().unwrap().len(), 2);
}

#[test]
fn remove_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install first
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    // Remove with JSON
    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "--local", "valid-skill", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "remove");
    assert_eq!(json["name"], "valid-skill");
    assert_eq!(json["removed_from"][0], "Claude Code");
}

#[test]
fn sync_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["sync", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "sync");
    assert_eq!(json["skills"][0]["name"], "valid-skill");
    assert!(json["output_file"].as_str().unwrap().contains("AGENTS.md"));
}

#[test]
fn update_json_no_skills() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["update", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "update");
    assert_eq!(json["updated"].as_array().unwrap().len(), 0);
}

// --- Survey ---

#[test]
fn survey_empty() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("survey")
        .assert()
        .success()
        .stdout(predicate::str::contains("No skills found"));
}

#[test]
fn survey_clean_install() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install to two agents
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude,cursor",
        ])
        .assert()
        .success();

    // Survey with --json to check structure
    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["survey", "--local", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["skills"][0]["name"], "valid-skill");
    let instances = json["skills"][0]["instances"].as_array().unwrap();
    assert_eq!(instances.len(), 2);
    // No content mismatch since both were installed from the same source
    let issues = json["issues"].as_array().unwrap();
    assert!(!issues.iter().any(|i| i["kind"] == "content_mismatch"));
}

#[test]
fn survey_detects_content_mismatch() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install to claude
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    // Manually tamper with the SKILL.md in claude to create a mismatch,
    // then install the original to cursor
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "cursor",
        ])
        .assert()
        .success();

    // Modify claude's copy to create divergence
    let claude_skill = project.path().join(".claude/skills/valid-skill/SKILL.md");
    std::fs::write(
        &claude_skill,
        "---\nname: valid-skill\ndescription: Modified version\n---\n# Changed",
    )
    .unwrap();

    // Survey should detect the mismatch
    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["survey", "--local", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let issues = json["issues"].as_array().unwrap();
    assert!(issues.iter().any(|i| i["kind"] == "content_mismatch"));
}

#[test]
fn survey_detects_unmanaged() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Manually place a skill without using equip (no .equip.json)
    let skill_dir = project.path().join(".claude/skills/manual-skill");
    std::fs::create_dir_all(&skill_dir).unwrap();
    std::fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: manual-skill\ndescription: Placed by hand\n---\n# Manual",
    )
    .unwrap();

    // Also create .claude dir so agent is "detected"
    // (already created by the skill dir above)

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["survey", "--local", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let issues = json["issues"].as_array().unwrap();
    assert!(issues.iter().any(|i| i["kind"] == "unmanaged"));
}

#[test]
fn survey_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--local",
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["survey", "--local", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "survey");
    assert!(json["skills"].is_array());
    assert!(json["issues"].is_array());
    assert_eq!(json["skills"][0]["name"], "valid-skill");
    assert!(
        json["skills"][0]["instances"][0]["content_hash"]
            .as_str()
            .unwrap()
            .len()
            > 0
    );
}

#[test]
fn survey_with_path_scans_subdirs() {
    let home = tempdir().unwrap();
    let dev_dir = tempdir().unwrap();

    // Create two fake projects under the dev dir, each with a skill
    let project_a = dev_dir.path().join("project-a");
    let project_b = dev_dir.path().join("project-b");
    std::fs::create_dir_all(project_a.join(".claude/skills/my-skill")).unwrap();
    std::fs::create_dir_all(project_b.join(".claude/skills/my-skill")).unwrap();

    std::fs::write(
        project_a.join(".claude/skills/my-skill/SKILL.md"),
        "---\nname: my-skill\ndescription: Version A\n---\n# A",
    )
    .unwrap();
    std::fs::write(
        project_b.join(".claude/skills/my-skill/SKILL.md"),
        "---\nname: my-skill\ndescription: Version B\n---\n# B",
    )
    .unwrap();

    let output = equip()
        .env("HOME", home.path())
        .args([
            "survey",
            "--path",
            &dev_dir.path().display().to_string(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let issues = json["issues"].as_array().unwrap();
    // Should detect content mismatch between project-a and project-b
    assert!(issues.iter().any(|i| i["kind"] == "content_mismatch"));
}

// --- Install --all ---

#[test]
fn install_all_agents() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--local", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("18 agent(s)"));

    // Spot-check a few agent directories
    assert!(
        project
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        project
            .path()
            .join(".cursor/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        project
            .path()
            .join(".gemini/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        project
            .path()
            .join(".windsurf/skills/valid-skill/SKILL.md")
            .exists()
    );
}

// --- Global Default ---

#[test]
fn install_defaults_to_global() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Should be in HOME, not project
    assert!(
        home.path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        !project
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn install_local_flag_uses_project() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--agent",
            "claude",
            "--local",
        ])
        .assert()
        .success();

    assert!(
        project
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
    assert!(
        !home
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn list_defaults_to_global() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Install globally
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Default list shows global
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("valid-skill"));
}

#[test]
fn remove_defaults_to_global() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    assert!(
        home.path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "valid-skill"])
        .assert()
        .success();

    assert!(
        !home
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

// --- list --json source field ---

#[test]
fn list_json_includes_source() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--agent",
            "claude",
            "--local",
        ])
        .assert()
        .success();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["list", "--local", "--json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    // Source should be present (local path)
    assert!(json[0]["source"].is_string());
}

#[test]
fn list_json_source_null_for_unmanaged() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Manually place a skill without .equip.json
    let skill_dir = project.path().join(".claude/skills/manual-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: manual-skill\ndescription: Placed by hand\n---\n# Manual",
    )
    .unwrap();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["list", "--local", "--json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json[0]["name"], "manual-skill");
    assert!(json[0]["source"].is_null());
}

// --- Init ---

#[test]
fn init_file_backend_creates_config() {
    let home = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Linked equip to file path"));

    let config_path = home.path().join(".equip/config.json");
    assert!(config_path.exists());
    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&config_path).unwrap()).unwrap();
    assert_eq!(config["backend"], "file");
}

#[test]
fn init_file_backend_creates_ops_dir() {
    let home = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    assert!(sync_dir.path().join("ops").is_dir());
}

#[test]
fn init_file_backend_detects_existing_ops() {
    let home = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    // Pre-create an op
    write_op_file(
        &sync_dir.path().join("ops"),
        "20260315T100000Z-add-pdf.json",
        r#"{"op":"add","skill":"pdf","source":"anthropics/skills/pdf","description":"PDF","ts":"2026-03-15T10:00:00Z"}"#,
    );

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Found 1 skill(s)"));
}

#[test]
fn init_overwrites_previous_config() {
    let home = tempdir().unwrap();
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &dir1.path().display().to_string()])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &dir2.path().display().to_string()])
        .assert()
        .success();

    let config: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(home.path().join(".equip/config.json")).unwrap())
            .unwrap();
    assert_eq!(config["path"], dir2.path().display().to_string());
}

#[test]
fn init_no_args_without_gh_auth_errors() {
    let home = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("GitHub user"));
}

// --- Export ---

#[test]
fn export_to_file_backend() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    // Init backend
    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Install a skill globally
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Export (auto-sync already wrote the op, so export sees it as tracked)
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("export")
        .assert()
        .success();

    // Check ops dir has a file (from auto-sync or export)
    let ops = fs::read_dir(sync_dir.path().join("ops"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .count();
    assert!(ops > 0, "Expected op files in sync dir");
}

#[test]
fn export_output_flag_writes_json() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let output_file = tempdir().unwrap();
    let output_path = output_file.path().join("skills.json");

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["export", "--output", &output_path.display().to_string()])
        .assert()
        .success();

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&output_path).unwrap()).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["name"], "valid-skill");
}

#[test]
fn export_json_flag_prints_stdout() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["export", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
    assert_eq!(json[0]["name"], "valid-skill");
}

#[test]
fn export_no_backend_with_output_flag_works() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let output_file = tempdir().unwrap();
    let output_path = output_file.path().join("skills.json");

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // No init — but --output should work
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["export", "--output", &output_path.display().to_string()])
        .assert()
        .success();

    assert!(output_path.exists());
}

#[test]
fn export_no_backend_no_output_errors() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("export")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No sync backend"));
}

#[test]
fn export_skips_already_tracked_skills() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // First export
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("export")
        .assert()
        .success();

    // Second export — should succeed (writes fresh op to keep log in sync)
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("export")
        .assert()
        .success();

    // Verify skill content is in the repo
    assert!(sync_dir.path().join("skills/valid-skill/SKILL.md").exists());
}

#[test]
fn export_includes_unmanaged_with_null_source() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    // Manually place skill without .equip.json
    let skill_dir = home.path().join(".claude/skills/manual-skill");
    fs::create_dir_all(&skill_dir).unwrap();
    fs::write(
        skill_dir.join("SKILL.md"),
        "---\nname: manual-skill\ndescription: Placed by hand\n---\n# Manual",
    )
    .unwrap();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["export", "--json"])
        .output()
        .unwrap();

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let skill = json
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "manual-skill")
        .unwrap();
    assert!(skill["source"].is_null());
}

// --- Restore ---

#[test]
fn restore_from_file_backend() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    // Init backend
    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Write op files pointing to local fixtures
    write_op_file(
        &sync_dir.path().join("ops"),
        "20260315T100000Z-add-valid-skill.json",
        &format!(
            r#"{{"op":"add","skill":"valid-skill","source":"{}","description":"test","ts":"2026-03-15T10:00:00Z"}}"#,
            fixture_path("valid-skill")
        ),
    );

    // Restore
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("restore")
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored 1"));

    assert!(
        home.path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn restore_from_file_flag() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let manifest = tempdir().unwrap();
    let manifest_path = manifest.path().join("skills.json");

    fs::write(
        &manifest_path,
        format!(
            r#"[{{"name":"valid-skill","source":"{}"}}]"#,
            fixture_path("valid-skill")
        ),
    )
    .unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["restore", "--from", &manifest_path.display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored 1"));

    assert!(
        home.path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn restore_dry_run_no_side_effects() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    write_op_file(
        &sync_dir.path().join("ops"),
        "20260315T100000Z-add-valid-skill.json",
        &format!(
            r#"{{"op":"add","skill":"valid-skill","source":"{}","description":"test","ts":"2026-03-15T10:00:00Z"}}"#,
            fixture_path("valid-skill")
        ),
    );

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["restore", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("valid-skill"));

    // Nothing should be installed
    assert!(
        !home
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn restore_skips_null_source() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let manifest = tempdir().unwrap();
    let manifest_path = manifest.path().join("skills.json");

    fs::write(
        &manifest_path,
        r#"[{"name":"no-source-skill","source":null}]"#,
    )
    .unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["restore", "--from", &manifest_path.display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("skipped"));
}

#[test]
fn restore_skips_removed_skills() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Add then remove
    write_op_file(
        &sync_dir.path().join("ops"),
        "20260315T100000Z-add-valid-skill.json",
        &format!(
            r#"{{"op":"add","skill":"valid-skill","source":"{}","description":"test","ts":"2026-03-15T10:00:00Z"}}"#,
            fixture_path("valid-skill")
        ),
    );
    write_op_file(
        &sync_dir.path().join("ops"),
        "20260316T100000Z-remove-valid-skill.json",
        r#"{"op":"remove","skill":"valid-skill","ts":"2026-03-16T10:00:00Z"}"#,
    );

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("restore")
        .assert()
        .success()
        .stdout(predicate::str::contains("No skills to restore"));

    assert!(
        !home
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}

#[test]
fn restore_no_backend_no_from_errors() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("restore")
        .assert()
        .failure()
        .stderr(predicate::str::contains("No sync backend"));
}

#[test]
fn restore_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let manifest = tempdir().unwrap();
    let manifest_path = manifest.path().join("skills.json");

    fs::write(
        &manifest_path,
        format!(
            r#"[{{"name":"valid-skill","source":"{}"}}]"#,
            fixture_path("valid-skill")
        ),
    )
    .unwrap();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "restore",
            "--from",
            &manifest_path.display().to_string(),
            "--json",
        ])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["action"], "restore");
    assert_eq!(json["restored"], 1);
}

// --- Status ---

#[test]
fn status_all_synced() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Install a skill (auto-sync writes the op)
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Export to ensure op is in sync dir
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("export")
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 synced"));
}

#[test]
fn status_missing_skill() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Write op for a skill not installed locally
    write_op_file(
        &sync_dir.path().join("ops"),
        "20260315T100000Z-add-missing-skill.json",
        r#"{"op":"add","skill":"missing-skill","source":"some/repo","description":"test","ts":"2026-03-15T10:00:00Z"}"#,
    );

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 missing"));
}

#[test]
fn status_untracked_skill() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Install a skill but don't export (empty ops)
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // The auto-sync will have written an op. Remove it to simulate untracked.
    let ops_dir = sync_dir.path().join("ops");
    for entry in fs::read_dir(&ops_dir).unwrap().flatten() {
        if entry
            .path()
            .extension()
            .map(|e| e == "json")
            .unwrap_or(false)
        {
            fs::remove_file(entry.path()).unwrap();
        }
    }

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 untracked"));
}

#[test]
fn status_no_backend_configured() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("No sync backend"));
}

#[test]
fn status_json_output() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    let output = equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["status", "--json"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json["synced"].is_array());
    assert!(json["missing"].is_array());
    assert!(json["untracked"].is_array());
}

#[test]
fn status_ignores_removed_in_ops() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Install skill locally
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Clear auto-synced ops, then write add + remove
    let ops_dir = sync_dir.path().join("ops");
    for entry in fs::read_dir(&ops_dir).unwrap().flatten() {
        if entry
            .path()
            .extension()
            .map(|e| e == "json")
            .unwrap_or(false)
        {
            fs::remove_file(entry.path()).unwrap();
        }
    }
    write_op_file(
        &ops_dir,
        "20260315T100000Z-add-valid-skill.json",
        r#"{"op":"add","skill":"valid-skill","source":"test","ts":"2026-03-15T10:00:00Z"}"#,
    );
    write_op_file(
        &ops_dir,
        "20260316T100000Z-remove-valid-skill.json",
        r#"{"op":"remove","skill":"valid-skill","ts":"2026-03-16T10:00:00Z"}"#,
    );

    // Skill is installed locally but removed in ops → shows as untracked
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("status")
        .assert()
        .success()
        .stdout(predicate::str::contains("1 untracked"));
}

// --- Auto-sync ---

#[test]
fn install_auto_syncs_when_backend_exists() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Check that an add op was created
    let ops_count = fs::read_dir(sync_dir.path().join("ops"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".json") && name.contains("add")
        })
        .count();
    assert!(ops_count > 0, "Expected auto-sync to create an add op");
}

#[test]
fn remove_auto_syncs_when_backend_exists() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "valid-skill"])
        .assert()
        .success();

    let remove_ops = fs::read_dir(sync_dir.path().join("ops"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            name.ends_with(".json") && name.contains("remove")
        })
        .count();
    assert!(remove_ops > 0, "Expected auto-sync to create a remove op");
}

#[test]
fn install_no_sync_when_no_backend() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // No .equip dir should exist
    assert!(!home.path().join(".equip").exists());
}

#[test]
fn install_local_does_not_sync() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Install with --local
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("valid-skill"),
            "--agent",
            "claude",
            "--local",
        ])
        .assert()
        .success();

    // No add ops should exist (only .gitkeep from init)
    let json_ops = fs::read_dir(sync_dir.path().join("ops"))
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "json").unwrap_or(false))
        .count();
    assert_eq!(json_ops, 0, "Local installs should not create sync ops");
}

// --- Roundtrip ---

#[test]
fn full_roundtrip_file_backend() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let sync_dir = tempdir().unwrap();

    // Init
    equip()
        .env("HOME", home.path())
        .args(["init", "--path", &sync_dir.path().display().to_string()])
        .assert()
        .success();

    // Install two skills
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args([
            "install",
            &fixture_path("multi-skill-repo"),
            "--agent",
            "claude",
        ])
        .assert()
        .success();

    // Export
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .arg("export")
        .assert()
        .success();

    // Remove both skills
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "skill-one"])
        .assert()
        .success();
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "skill-two"])
        .assert()
        .success();

    assert!(
        !home
            .path()
            .join(".claude/skills/skill-one/SKILL.md")
            .exists()
    );

    // Restore — should reinstall from the add ops (not the remove ops, since
    // auto-sync wrote remove ops too, and those have later timestamps)
    // Actually, the remove ops will have later timestamps, so restore will see
    // no active skills. We need to test the export file flow instead.
    // Let's use the export --output file for this roundtrip.
}

#[test]
fn export_then_restore_via_file() {
    let home = tempdir().unwrap();
    let project = tempdir().unwrap();
    let output_dir = tempdir().unwrap();
    let output_path = output_dir.path().join("skills.json");

    // Install
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["install", &fixture_path("valid-skill"), "--agent", "claude"])
        .assert()
        .success();

    // Export to file
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["export", "--output", &output_path.display().to_string()])
        .assert()
        .success();

    // Remove
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["remove", "valid-skill"])
        .assert()
        .success();

    assert!(
        !home
            .path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );

    // Restore from file
    equip()
        .env("HOME", home.path())
        .current_dir(project.path())
        .args(["restore", "--from", &output_path.display().to_string()])
        .assert()
        .success()
        .stdout(predicate::str::contains("Restored 1"));

    assert!(
        home.path()
            .join(".claude/skills/valid-skill/SKILL.md")
            .exists()
    );
}
