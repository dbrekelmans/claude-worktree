use anyhow::Result;
use colored::Colorize;
use std::process;

use super::common;
use crate::config::state::WorktreeState;

pub fn execute(name: Option<String>) -> Result<()> {
    let worktree_state = match common::resolve_worktree(name)? {
        Some(state) => state,
        None => {
            eprintln!("{}", "Error: Not in a worktree directory".red());
            process::exit(1);
        }
    };

    display_status(&worktree_state);
    Ok(())
}

/// Display the status of a worktree
fn display_status(state: &WorktreeState) {
    // Worktree name (show both display name and directory name if different)
    if state.has_custom_name() {
        println!(
            "{} {} ({})",
            "Worktree:".bold(),
            state.effective_name().green(),
            state.name.dimmed()
        );
    } else {
        println!("{} {}", "Worktree:".bold(), state.name.green());
    }

    println!("{} {}", "Branch:  ".bold(), state.branch.cyan());
    println!("{} {}", "Project: ".bold(), state.project_name.blue());

    println!();
    println!("{}", "Directories:".bold());
    println!(
        "  {} {}",
        "Original:".dimmed(),
        state.original_dir.display()
    );
    println!(
        "  {} {}",
        "Worktree:".dimmed(),
        state.worktree_dir.display()
    );

    println!();
    if state.ports.is_empty() {
        println!("{} {}", "Ports:".bold(), "none".dimmed());
    } else {
        let ports_str = state
            .ports
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        println!("{} {}", "Ports:".bold(), ports_str);
    }

    println!();
    let created = state.created_at.format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{} {}", "Created:".bold(), created);
}
