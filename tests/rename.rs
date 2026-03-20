mod helpers;

use helpers::TestEnv;
use predicates::prelude::*;

#[test]
fn test_rename_updates_display_name() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    // Rename the worktree
    env.cmd()
        .args(["rename", "my-feature", &name])
        .assert()
        .success();

    // Read state.json and verify displayName
    let worktree_dir = env.find_worktree_dir(&name);
    let state_path = worktree_dir.join("state.json");
    let state: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&state_path).unwrap()).unwrap();

    assert_eq!(state["displayName"], "my-feature");
}

#[test]
fn test_rename_clear() {
    let env = TestEnv::new();
    let name = env.create_worktree();
    let worktree_dir = env.find_worktree_dir(&name);

    // Set a display name first
    env.cmd()
        .args(["rename", "my-feature", &name])
        .assert()
        .success();

    // Clear the display name (run from inside worktree so it auto-detects context)
    env.cmd_in(&worktree_dir)
        .args(["rename", "--clear"])
        .assert()
        .success();

    // Read state.json and verify displayName is null/absent
    let state_path = worktree_dir.join("state.json");
    let state: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&state_path).unwrap()).unwrap();

    assert!(
        state.get("displayName").is_none() || state["displayName"].is_null(),
        "displayName should be null or absent after --clear, got: {:?}",
        state.get("displayName")
    );
}

#[test]
fn test_rename_invalid_chars() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    // Attempt to rename with invalid characters (path separator)
    env.cmd()
        .args(["rename", "foo/bar", &name])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("path separator")
                .or(predicate::str::contains("path separator")),
        );
}
