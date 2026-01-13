use std::path::PathBuf;

/// Returns the global worktree directory (~/.worktree/)
pub fn global_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not determine home directory")
        .join(".worktree")
}

/// Returns the global worktrees storage directory (~/.worktree/worktrees/)
pub fn global_worktrees_dir() -> PathBuf {
    global_dir().join("worktrees")
}

/// Returns the port allocations file path (~/.worktree/port-allocations.json)
pub fn allocations_file() -> PathBuf {
    global_dir().join("port-allocations.json")
}

/// Returns the project config directory (.worktree/)
pub fn project_config_dir() -> PathBuf {
    PathBuf::from(".worktree")
}

/// Returns the project config directory relative to a given root
pub fn project_config_dir_in(root: &std::path::Path) -> PathBuf {
    root.join(".worktree")
}

/// Returns the settings file path (.worktree/settings.json)
pub fn settings_file() -> PathBuf {
    project_config_dir().join("settings.json")
}

/// Returns the settings file path relative to a given root
pub fn settings_file_in(root: &std::path::Path) -> PathBuf {
    project_config_dir_in(root).join("settings.json")
}

/// Returns the local settings file path (.worktree/settings.local.json)
pub fn local_settings_file() -> PathBuf {
    project_config_dir().join("settings.local.json")
}

/// Returns the local settings file path relative to a given root
pub fn local_settings_file_in(root: &std::path::Path) -> PathBuf {
    project_config_dir_in(root).join("settings.local.json")
}

/// Returns the setup script path (.worktree/setup.sh)
pub fn setup_script() -> PathBuf {
    project_config_dir().join("setup.sh")
}

/// Returns the run script path (.worktree/run.sh)
pub fn run_script() -> PathBuf {
    project_config_dir().join("run.sh")
}

/// Returns the stop script path (.worktree/stop.sh)
pub fn stop_script() -> PathBuf {
    project_config_dir().join("stop.sh")
}

/// Returns the close script path (.worktree/close.sh)
pub fn close_script() -> PathBuf {
    project_config_dir().join("close.sh")
}

/// Returns the SETUP.md file path (.worktree/SETUP.md)
pub fn setup_md() -> PathBuf {
    project_config_dir().join("SETUP.md")
}

/// Ensures the global directory exists
pub fn ensure_global_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(global_dir())
}

/// Ensures the global worktrees directory exists
pub fn ensure_global_worktrees_dir() -> std::io::Result<()> {
    std::fs::create_dir_all(global_worktrees_dir())
}
