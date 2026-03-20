use anyhow::{bail, Result};
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};
use crate::git;
use crate::ports;
use crate::scripts;

/// Options for worktree removal
#[derive(Default)]
pub struct RemoveOptions {
    /// Whether to print verbose output
    pub verbose: bool,
}

/// Result of a worktree removal operation
pub struct RemoveResult {
    /// Whether the close script ran successfully
    pub close_script_success: Option<bool>,
    /// The ports that were deallocated, if any
    pub deallocated_ports: Option<Vec<u16>>,
    /// Whether the git worktree was removed successfully
    pub worktree_removed: bool,
}

/// Remove a worktree: run close script, deallocate ports, and remove the git worktree
/// This is the shared logic between the `close` and `cleanup` commands.
pub fn remove_worktree(state: &WorktreeState, options: &RemoveOptions) -> Result<RemoveResult> {
    let mut result = RemoveResult {
        close_script_success: None,
        deallocated_ports: None,
        worktree_removed: false,
    };

    // Run close script if it exists
    let close_script = state.worktree_dir.join(".worktree").join("close.sh");
    if close_script.exists() {
        if options.verbose {
            println!("  Running close script...");
        }
        let env = scripts::build_env_vars(state);
        let success = scripts::execute_script_ignore_errors(&close_script, &env);
        result.close_script_success = Some(success);
    }

    // Deallocate ports
    if options.verbose {
        println!("  Deallocating ports...");
    }
    match ports::deallocate(&state.allocation_key) {
        Ok(ports) => {
            result.deallocated_ports = ports;
        }
        Err(_) => {
            // Ignore errors during port deallocation
        }
    }

    // Remove git worktree
    if options.verbose {
        println!("  Removing git worktree...");
    }
    match git::remove_worktree(&state.original_dir, &state.worktree_dir, true) {
        Ok(_) => {
            result.worktree_removed = true;
        }
        Err(_) => {
            // Try manual removal as fallback
            if std::fs::remove_dir_all(&state.worktree_dir).is_ok() {
                result.worktree_removed = true;
            }
        }
    }

    Ok(result)
}

/// Find all worktrees in the global directory, sorted newest-first
pub fn find_all_worktrees() -> Result<Vec<WorktreeState>> {
    let mut worktrees = Vec::new();
    let base_dir = paths::global_worktrees_dir()?;

    if !base_dir.exists() {
        return Ok(worktrees);
    }

    for entry in WalkDir::new(&base_dir)
        .min_depth(1)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_name() == "state.json" {
            if let Ok(state) = WorktreeState::load(entry.path()) {
                worktrees.push(state);
            }
        }
    }

    worktrees.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(worktrees)
}

/// Try to get the current project name from the git repo or worktree state
pub fn get_current_project() -> Option<String> {
    if let Ok(Some(state)) = crate::config::state::detect_worktree() {
        return Some(state.project_name);
    }

    if git::is_git_repo() {
        if let Ok(name) = git::get_main_project_name() {
            return Some(name);
        }
    }

    None
}

/// Find worktrees for the current project, or all if not in a project
pub fn find_worktrees_for_current_project() -> Result<Vec<WorktreeState>> {
    let mut worktrees = find_all_worktrees()?;

    if let Some(project) = get_current_project() {
        worktrees.retain(|wt| wt.project_name == project);
    }

    Ok(worktrees)
}

/// Resolve a worktree by name identifier, falling back to detecting the current worktree
pub fn resolve_worktree(name: Option<String>) -> Result<Option<WorktreeState>> {
    if let Some(name) = name {
        let all_worktrees = find_all_worktrees()?;
        let matches: Vec<_> = all_worktrees
            .into_iter()
            .filter(|wt| wt.matches_identifier(&name))
            .collect();

        match matches.len() {
            0 => bail!("No worktree found with name '{}'", name),
            1 => return Ok(Some(matches.into_iter().next().unwrap())),
            _ => bail!(
                "Multiple worktrees match '{}'. Please be more specific.",
                name
            ),
        }
    }

    crate::config::state::detect_worktree()
}
