mod helpers;

use helpers::TestEnv;
#[test]
fn test_new_creates_worktree_directory() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let worktree_dir = env.find_worktree_dir(&name);
    assert!(
        worktree_dir.exists(),
        "Worktree directory should exist at {:?}",
        worktree_dir
    );
}

#[test]
fn test_new_creates_state_json() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let state_path = env.find_worktree_dir(&name).join("state.json");
    assert!(state_path.exists(), "state.json should exist");

    let contents = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let state: serde_json::Value =
        serde_json::from_str(&contents).expect("state.json should be valid JSON");

    assert_eq!(
        state["projectName"].as_str().unwrap(),
        "project",
        "projectName should be 'project'"
    );
    assert!(
        state["branch"].as_str().unwrap().starts_with("worktree/"),
        "branch should start with 'worktree/'"
    );
    let ports = state["ports"].as_array().expect("ports should be an array");
    assert!(!ports.is_empty(), "ports should not be empty");
}

#[test]
fn test_new_allocates_ports() {
    let env = TestEnv::new();
    let _name = env.create_worktree();

    let alloc_path = env.port_allocations_path();
    assert!(alloc_path.exists(), "port-allocations.json should exist");

    let contents =
        std::fs::read_to_string(&alloc_path).expect("failed to read port-allocations.json");
    let allocations: serde_json::Value =
        serde_json::from_str(&contents).expect("port-allocations.json should be valid JSON");

    let obj = allocations
        .as_object()
        .expect("allocations should be an object");
    assert!(
        !obj.is_empty(),
        "port allocations should have at least one entry"
    );
}

#[test]
fn test_new_creates_git_branch() {
    let env = TestEnv::new();
    let name = env.create_worktree();

    let output = env.git(&["branch", "--list", &format!("worktree/{}", name)]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.trim().is_empty(),
        "git branch should list the worktree branch 'worktree/{}'",
        name
    );
}

#[test]
fn test_new_with_param() {
    let env = TestEnv::new();
    env.init_project();

    let output = env
        .cmd()
        .args(["new", "my-feature"])
        .output()
        .expect("failed to run new");
    assert!(
        output.status.success(),
        "new my-feature should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let name = TestEnv::worktree_name_from_output(&stdout);

    let state_path = env.find_worktree_dir(&name).join("state.json");
    let contents = std::fs::read_to_string(&state_path).expect("failed to read state.json");
    let state: serde_json::Value =
        serde_json::from_str(&contents).expect("state.json should be valid JSON");

    assert_eq!(
        state["param"].as_str().unwrap(),
        "my-feature",
        "param should be 'my-feature'"
    );
    assert_eq!(
        state["displayName"].as_str().unwrap(),
        "my-feature",
        "displayName should be 'my-feature'"
    );
}

#[test]
fn test_new_not_in_git_repo_fails() {
    let env = TestEnv::new();
    let non_git_dir = tempfile::TempDir::new().expect("failed to create temp dir");

    env.cmd_in(non_git_dir.path())
        .args(["new"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("Not in a git repository"));
}
