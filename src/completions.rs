use clap_complete::engine::CompletionCandidate;

use crate::commands::common;

/// Get worktree name completion candidates
/// Returns all worktree names, optionally filtered by current project
pub fn worktree_names() -> Vec<CompletionCandidate> {
    let worktrees = match common::find_all_worktrees() {
        Ok(wts) => wts,
        Err(_) => return vec![],
    };

    // Try to get the current project to filter results
    let current_project = common::get_current_project();

    worktrees
        .into_iter()
        .filter(|wt| {
            // If we know the current project, only show worktrees from that project
            current_project
                .as_ref()
                .map(|p| &wt.project_name == p)
                .unwrap_or(true)
        })
        .flat_map(|wt| {
            let mut candidates = vec![CompletionCandidate::new(&wt.name)
                .help(Some(format!("{} [{}]", wt.project_name, wt.branch).into()))];

            // Also add display name as a completion candidate if it differs
            if let Some(ref display_name) = wt.display_name {
                if display_name != &wt.name {
                    candidates.push(
                        CompletionCandidate::new(display_name)
                            .help(Some(format!("{} [{}]", wt.project_name, wt.branch).into())),
                    );
                }
            }

            candidates
        })
        .collect()
}
