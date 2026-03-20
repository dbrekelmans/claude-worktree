mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_status_by_name() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    env.cmd()
        .args(["status", &name])
        .assert()
        .success()
        .stdout(predicate::str::contains(&name))
        .stdout(predicate::str::contains("Branch:"))
        .stdout(predicate::str::contains("Project:"))
        .stdout(predicate::str::contains("Ports:"));
}

#[test]
fn test_status_outside_worktree_fails() {
    let env = TestEnv::new();
    env.cmd()
        .args(["status"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a worktree"));
}
