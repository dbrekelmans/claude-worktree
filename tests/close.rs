mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_close_removes_worktree_dir() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let worktree_dir = env.find_worktree_dir(&name);
    assert!(
        worktree_dir.exists(),
        "Worktree dir should exist before close"
    );

    env.cmd()
        .args(["close", "--force", &name])
        .assert()
        .success();

    assert!(
        !worktree_dir.exists(),
        "Worktree dir should be removed after close"
    );
}

#[test]
fn test_close_removes_port_allocation() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let alloc_path = env.port_allocations_path();
    assert!(alloc_path.exists(), "port-allocations.json should exist");

    let contents =
        std::fs::read_to_string(&alloc_path).expect("failed to read port-allocations.json");
    let before: serde_json::Value = serde_json::from_str(&contents).expect("should be valid JSON");
    assert!(
        !before.as_object().unwrap().is_empty(),
        "port allocations should have entries before close"
    );

    env.cmd()
        .args(["close", "--force", &name])
        .assert()
        .success();

    let contents =
        std::fs::read_to_string(&alloc_path).expect("failed to read port-allocations.json");
    let after: serde_json::Value = serde_json::from_str(&contents).expect("should be valid JSON");
    let obj = after.as_object().unwrap();
    assert!(
        obj.is_empty(),
        "port allocations should be empty after close, got: {:?}",
        obj
    );
}

#[test]
fn test_close_removes_git_worktree_ref() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    // Verify git knows about the worktree before close
    let output = env.git(&["worktree", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(&name),
        "git worktree list should contain the worktree before close"
    );

    env.cmd()
        .args(["close", "--force", &name])
        .assert()
        .success();

    // Verify git no longer lists the worktree after close
    let output = env.git(&["worktree", "list"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(&name),
        "git worktree list should not contain the worktree after close"
    );
}

#[test]
fn test_close_nonexistent_fails() {
    let env = TestEnv::new();
    env.init_project();

    env.cmd()
        .args(["close", "--force", "nonexistent"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No worktree found"));
}
