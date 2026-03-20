use anyhow::Result;
use colored::Colorize;
use std::process;

use super::common;

pub fn execute(name: Option<String>) -> Result<()> {
    let worktree_state = match common::resolve_worktree(name)? {
        Some(state) => state,
        None => {
            eprintln!("{}", "Error: Not in a worktree directory".red());
            process::exit(1);
        }
    };

    println!("{}", worktree_state.worktree_dir.display());
    Ok(())
}
