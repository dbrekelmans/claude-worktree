mod helpers;

use helpers::TestEnv;

#[test]
fn test_completions_bash() {
    let env = TestEnv::new();

    let output = env
        .cmd()
        .args(["completions", "bash"])
        .output()
        .expect("failed to run completions command");

    assert!(
        output.status.success(),
        "completions command should succeed"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty(), "completions output should not be empty");
}
