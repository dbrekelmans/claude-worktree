mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_path_prints_worktree_dir() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let expected_dir = env.find_worktree_dir(&name);

    let output = env
        .cmd()
        .args(["path", &name])
        .output()
        .expect("failed to run path");
    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    let printed_path = stdout.trim();

    assert!(
        !printed_path.is_empty(),
        "path command should print a non-empty path"
    );
    assert_eq!(
        std::path::Path::new(printed_path),
        expected_dir,
        "printed path should match the expected worktree directory"
    );
    assert!(
        std::path::Path::new(printed_path).exists(),
        "printed path should exist on disk"
    );
}

#[test]
fn test_path_outside_worktree_fails() {
    let env = TestEnv::new();
    env.cmd()
        .args(["path"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Not in a worktree"));
}
