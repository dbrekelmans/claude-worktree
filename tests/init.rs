mod helpers;
use helpers::TestEnv;
use predicates::prelude::*;
use std::os::unix::fs::PermissionsExt;

#[test]
fn test_init_creates_settings() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-scripts", "--no-ai"])
        .assert()
        .success();

    let settings_path = env.repo_path().join(".worktree/settings.json");
    assert!(
        settings_path.exists(),
        "settings.json should exist after init"
    );
}

#[test]
fn test_init_creates_all_files() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-ai"])
        .assert()
        .success();

    let worktree_dir = env.repo_path().join(".worktree");
    let expected_files = [
        "settings.json",
        ".gitignore",
        "README.md",
        "SETUP.md",
        "setup.sh",
        "run.sh",
        "stop.sh",
        "close.sh",
    ];

    for file in &expected_files {
        let path = worktree_dir.join(file);
        assert!(path.exists(), "{} should exist in .worktree/", file);
    }
}

#[test]
fn test_init_no_scripts_skips_scripts() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-scripts", "--no-ai"])
        .assert()
        .success();

    let worktree_dir = env.repo_path().join(".worktree");
    assert!(
        worktree_dir.join("settings.json").exists(),
        "settings.json should exist"
    );

    let script_files = ["setup.sh", "run.sh", "stop.sh", "close.sh"];
    for file in &script_files {
        let path = worktree_dir.join(file);
        assert!(
            !path.exists(),
            "{} should not exist with --no-scripts",
            file
        );
    }
}

#[test]
fn test_init_no_ai_creates_template_scripts() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-ai"])
        .assert()
        .success();

    let worktree_dir = env.repo_path().join(".worktree");
    let script_files = ["setup.sh", "run.sh", "stop.sh", "close.sh"];

    for file in &script_files {
        let path = worktree_dir.join(file);
        let content =
            std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to read {}", file));
        assert!(
            content.contains("TODO"),
            "{} should contain TODO template content",
            file
        );
    }
}

#[test]
fn test_init_scripts_are_executable() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-ai"])
        .assert()
        .success();

    let worktree_dir = env.repo_path().join(".worktree");
    let script_files = ["setup.sh", "run.sh", "stop.sh", "close.sh"];

    for file in &script_files {
        let path = worktree_dir.join(file);
        let metadata = std::fs::metadata(&path)
            .unwrap_or_else(|_| panic!("Failed to get metadata for {}", file));
        let mode = metadata.permissions().mode();
        assert_eq!(
            mode & 0o755,
            0o755,
            "{} should have executable permissions (0o755), got {:#o}",
            file,
            mode
        );
    }
}

#[test]
fn test_init_twice_fails() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-scripts", "--no-ai"])
        .assert()
        .success();

    env.cmd()
        .args(["init", "--defaults", "--no-scripts", "--no-ai"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_init_not_in_git_repo() {
    let env = TestEnv::new();
    let tmp_dir = tempfile::TempDir::new().expect("Failed to create temp dir");

    env.cmd_in(tmp_dir.path())
        .args(["init", "--defaults", "--no-scripts", "--no-ai"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a git repository"));
}

#[test]
fn test_init_settings_json_has_defaults() {
    let env = TestEnv::new();
    env.cmd()
        .args(["init", "--defaults", "--no-scripts", "--no-ai"])
        .assert()
        .success();

    let settings_path = env.repo_path().join(".worktree/settings.json");
    let content = std::fs::read_to_string(&settings_path).expect("Failed to read settings.json");
    let value: serde_json::Value =
        serde_json::from_str(&content).expect("Failed to parse settings.json");

    assert_eq!(value["portCount"], 10, "portCount should default to 10");
    assert_eq!(
        value["branchPrefix"], "worktree/",
        "branchPrefix should default to 'worktree/'"
    );
}
