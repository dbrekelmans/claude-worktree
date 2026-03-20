mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_list_empty() {
    let env = TestEnv::new();
    env.cmd()
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No active worktrees found."));
}

#[test]
fn test_list_json_empty() {
    let env = TestEnv::new();
    env.cmd()
        .args(["list", "--json", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn test_list_shows_worktree() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    env.cmd()
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&name));
}

#[test]
fn test_list_json_contains_worktree() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let output = env
        .cmd()
        .args(["list", "--json", "--all"])
        .output()
        .expect("failed to run list --json --all");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parsed: serde_json::Value = serde_json::from_str(stdout.trim()).expect("invalid JSON");
    let arr = parsed.as_array().expect("expected JSON array");
    assert_eq!(arr.len(), 1, "expected exactly one worktree in JSON output");
    assert_eq!(
        arr[0]["name"].as_str().expect("missing name field"),
        name,
        "worktree name should match"
    );
}

#[test]
fn test_list_all_flag() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    env.cmd()
        .args(["list", "--all"])
        .assert()
        .success()
        .stdout(predicate::str::contains(&name));
}
