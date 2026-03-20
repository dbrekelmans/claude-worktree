mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_run_executes_script() {
    let env = TestEnv::new();
    let name = env.create_worktree_with_scripts();
    let worktree_dir = env.find_worktree_dir(&name);

    env.cmd_in(&worktree_dir)
        .args(["run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Development environment started!"));
}

#[test]
fn test_run_outside_worktree_fails() {
    let env = TestEnv::new();

    env.cmd()
        .args(["run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a worktree"));
}

#[test]
fn test_stop_executes_script() {
    let env = TestEnv::new();
    let name = env.create_worktree_with_scripts();
    let worktree_dir = env.find_worktree_dir(&name);

    env.cmd_in(&worktree_dir)
        .args(["stop"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Services stopped."));
}

#[test]
fn test_stop_outside_worktree_fails() {
    let env = TestEnv::new();

    env.cmd()
        .args(["stop"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a worktree"));
}

#[test]
fn test_run_without_script_fails() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    env.cmd_in(&worktree_dir)
        .args(["run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Run script not found"));
}
