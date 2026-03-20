use assert_cmd::Command;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Isolated test environment with a fake HOME and a git repo.
pub struct TestEnv {
    /// Temp directory used as fake HOME — dropped last to clean up.
    _home_dir: TempDir,
    /// Path to the fake HOME.
    home: PathBuf,
    /// Path to the git repo inside the fake HOME.
    repo: PathBuf,
}

impl TestEnv {
    /// Create a new isolated test environment:
    /// - fake HOME with `~/.config/worktree/config.json` pre-seeded
    /// - `~/.worktree/` directory created
    /// - a git repo with one commit inside `$HOME/project`
    pub fn new() -> Self {
        let home_dir = TempDir::new().expect("failed to create temp dir");
        let home = home_dir.path().to_path_buf();

        // Pre-seed user config so ensure_configured() doesn't prompt
        let config_dir = home.join(".config").join("worktree");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(
            config_dir.join("config.json"),
            r#"{"autoLaunchTerminal": false}"#,
        )
        .unwrap();

        // Create global worktree directory
        std::fs::create_dir_all(home.join(".worktree")).unwrap();

        // Create a git repo with one commit
        let repo = home.join("project");
        std::fs::create_dir_all(&repo).unwrap();

        let run = |args: &[&str], dir: &Path| {
            std::process::Command::new("git")
                .args(args)
                .current_dir(dir)
                .env("HOME", &home)
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("GIT_AUTHOR_NAME", "Test")
                .env("GIT_AUTHOR_EMAIL", "test@test.com")
                .env("GIT_COMMITTER_NAME", "Test")
                .env("GIT_COMMITTER_EMAIL", "test@test.com")
                .output()
                .expect("failed to run git");
        };

        run(&["init", "-b", "main"], &repo);
        run(&["config", "user.name", "Test"], &repo);
        run(&["config", "user.email", "test@test.com"], &repo);
        std::fs::write(repo.join("README.md"), "# Test Project\n").unwrap();
        run(&["add", "."], &repo);
        run(&["commit", "-m", "initial commit"], &repo);

        Self {
            _home_dir: home_dir,
            home,
            repo,
        }
    }

    /// Get a `Command` for the worktree binary, with HOME set to the fake dir
    /// and current_dir set to the git repo.
    pub fn cmd(&self) -> Command {
        let mut cmd = Command::cargo_bin("worktree").expect("binary not found");
        cmd.env("HOME", &self.home)
            .env("NO_COLOR", "1")
            .current_dir(&self.repo);
        cmd
    }

    /// Get a `Command` for the worktree binary, with a custom current_dir.
    pub fn cmd_in(&self, dir: &Path) -> Command {
        let mut cmd = Command::cargo_bin("worktree").expect("binary not found");
        cmd.env("HOME", &self.home)
            .env("NO_COLOR", "1")
            .current_dir(dir);
        cmd
    }

    /// Get the path to the git repo.
    pub fn repo_path(&self) -> &Path {
        &self.repo
    }

    /// Expose home path for filesystem assertions.
    pub fn home(&self) -> &Path {
        &self.home
    }

    /// Path to `~/.worktree/port-allocations.json`.
    pub fn port_allocations_path(&self) -> PathBuf {
        self.home.join(".worktree").join("port-allocations.json")
    }

    /// Initialize the project with `--defaults --no-scripts --no-ai`.
    pub fn init_project(&self) {
        self.cmd()
            .args(["init", "--defaults", "--no-scripts", "--no-ai"])
            .assert()
            .success();
    }

    /// Initialize the project with `--defaults --no-ai` (generates template scripts).
    pub fn init_project_with_scripts(&self) {
        self.cmd()
            .args(["init", "--defaults", "--no-ai"])
            .assert()
            .success();
    }

    /// Run `init_project()` + `new`, parse worktree name from output, return it.
    pub fn create_worktree(&self) -> String {
        self.init_project();
        let output = self
            .cmd()
            .args(["new"])
            .output()
            .expect("failed to run new");
        assert!(
            output.status.success(),
            "new command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::worktree_name_from_output(&stdout)
    }

    /// Like `create_worktree` but uses `init_project_with_scripts()`.
    /// Commits the .worktree dir to git so scripts are available in the worktree.
    pub fn create_worktree_with_scripts(&self) -> String {
        self.init_project_with_scripts();
        // Commit .worktree so the scripts appear in worktree copies
        self.git(&["add", ".worktree"]);
        self.git(&["commit", "-m", "add worktree config"]);
        let output = self
            .cmd()
            .args(["new"])
            .output()
            .expect("failed to run new");
        assert!(
            output.status.success(),
            "new command failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        Self::worktree_name_from_output(&stdout)
    }

    /// Extract worktree name from `new` command output.
    /// The output contains a line like "Name: <name>" near the end.
    pub fn worktree_name_from_output(output: &str) -> String {
        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("Name:") {
                // "Name: swift-falcon-a3b2" or "Name: display-name - dir-name"
                let after_name = trimmed.trim_start_matches("Name:").trim();
                // If it contains " - ", the directory name is after the dash
                // But for our purposes we want the directory name (used for close, etc.)
                // The directory name is the last word, or the only word
                if let Some(dash_pos) = after_name.find(" - ") {
                    return after_name[dash_pos + 3..].trim().to_string();
                }
                return after_name.to_string();
            }
        }
        panic!("Could not find worktree name in output:\n{}", output);
    }

    /// Find the created worktree directory under `~/.worktree/worktrees/`.
    pub fn find_worktree_dir(&self, name: &str) -> PathBuf {
        let worktrees_dir = self
            .home
            .join(".worktree")
            .join("worktrees")
            .join("project");
        worktrees_dir.join(name)
    }

    /// Run a git command in the repo.
    pub fn git(&self, args: &[&str]) -> std::process::Output {
        std::process::Command::new("git")
            .args(args)
            .current_dir(&self.repo)
            .env("HOME", &self.home)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("failed to run git")
    }
}
