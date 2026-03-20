mod helpers;
use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_no_args_shows_help() {
    let env = TestEnv::new();
    env.cmd()
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage git worktrees"));
}

#[test]
fn test_version_flag() {
    let env = TestEnv::new();
    env.cmd()
        .args(["--version"])
        .assert()
        .success()
        .stdout(predicate::str::contains("worktree"));
}
