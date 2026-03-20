mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_dotenv_set_and_get() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Set a key
    env.cmd_in(&worktree_dir)
        .args(["dotenv", "set", "MY_KEY", "my_value"])
        .assert()
        .success();

    // Get the key and verify output
    env.cmd_in(&worktree_dir)
        .args(["dotenv", "get", "MY_KEY"])
        .assert()
        .success()
        .stdout(predicate::str::contains("my_value"));
}

#[test]
fn test_dotenv_set_creates_file() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Set a key
    env.cmd_in(&worktree_dir)
        .args(["dotenv", "set", "SOME_VAR", "some_value"])
        .assert()
        .success();

    // Verify .env file exists in the worktree directory
    let dotenv_path = worktree_dir.join(".env");
    assert!(
        dotenv_path.exists(),
        ".env file should exist at {:?}",
        dotenv_path
    );
}

#[test]
fn test_dotenv_get_missing_key() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Set one key
    env.cmd_in(&worktree_dir)
        .args(["dotenv", "set", "EXISTING_KEY", "value"])
        .assert()
        .success();

    // Try to get a different key that doesn't exist
    env.cmd_in(&worktree_dir)
        .args(["dotenv", "get", "MISSING_KEY"])
        .assert()
        .failure()
        .stderr(predicate::str::is_match("(?i)not found").unwrap());
}
