mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_cp_file_from_main_to_worktree() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Write a file in the main repo
    std::fs::write(env.repo_path().join("testfile.txt"), "hello").unwrap();

    // Copy it to the worktree (defaults: from=original repo, to=current worktree)
    env.cmd_in(&worktree_dir)
        .args(["cp", "testfile.txt"])
        .assert()
        .success();

    // Verify file was copied
    let dest = worktree_dir.join("testfile.txt");
    assert!(dest.exists(), "testfile.txt should exist in worktree");
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "hello");
}

#[test]
fn test_cp_skip_if_not_exists() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    env.cmd_in(&worktree_dir)
        .args(["cp", "nonexistent.txt", "--skip-if-not-exists"])
        .assert()
        .success();
}

#[test]
fn test_cp_create_if_not_exists() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    env.cmd_in(&worktree_dir)
        .args(["cp", "nonexistent.txt", "--create-if-not-exists"])
        .assert()
        .success();

    // Verify an empty file was created
    let dest = worktree_dir.join("nonexistent.txt");
    assert!(dest.exists(), "nonexistent.txt should have been created");
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "");
}

#[test]
fn test_cp_fails_if_dest_exists() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Write a file in the main repo
    std::fs::write(env.repo_path().join("testfile.txt"), "hello").unwrap();

    // Copy it once (should succeed)
    env.cmd_in(&worktree_dir)
        .args(["cp", "testfile.txt"])
        .assert()
        .success();

    // Copy again without --force (should fail)
    env.cmd_in(&worktree_dir)
        .args(["cp", "testfile.txt"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("already exists"));
}

#[test]
fn test_cp_force_overwrites() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Write v1 in the main repo and copy it
    std::fs::write(env.repo_path().join("testfile.txt"), "v1").unwrap();
    env.cmd_in(&worktree_dir)
        .args(["cp", "testfile.txt"])
        .assert()
        .success();

    // Update to v2 and force copy
    std::fs::write(env.repo_path().join("testfile.txt"), "v2").unwrap();
    env.cmd_in(&worktree_dir)
        .args(["cp", "testfile.txt", "--force"])
        .assert()
        .success();

    // Verify worktree has v2
    let dest = worktree_dir.join("testfile.txt");
    assert_eq!(std::fs::read_to_string(&dest).unwrap(), "v2");
}
