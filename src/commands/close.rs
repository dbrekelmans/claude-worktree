use anyhow::{bail, Result};
use colored::Colorize;
use std::io::{self, Write};
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};
use crate::git;
use crate::ports;
use crate::scripts;

pub fn execute(name: Option<String>, force: bool, interactive: bool) -> Result<()> {
    // Determine which worktree to close
    let worktree_state = resolve_worktree(name, interactive)?;

    println!(
        "{} {}/{}",
        "Closing:".bold(),
        worktree_state.project_name.blue(),
        worktree_state.name.green()
    );
    println!("  {} {}", "Path:".dimmed(), worktree_state.worktree_dir.display());

    // Confirm unless force flag is set
    if !force {
        print!(
            "\n{} ",
            "Are you sure you want to close this worktree? (y/N):".yellow()
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_lowercase();

        if input != "y" && input != "yes" {
            println!("{}", "Cancelled.".dimmed());
            return Ok(());
        }
    }

    println!();

    // Run close script if it exists (ignore errors)
    let close_script = worktree_state.worktree_dir.join(".worktree").join("close.sh");
    if close_script.exists() {
        println!("  Running close script...");
        let env = scripts::build_env_vars(&worktree_state);
        if scripts::execute_script_ignore_errors(&close_script, &env) {
            println!("  {} Close script completed", "✓".green());
        } else {
            println!("  {} Close script failed (continuing anyway)", "⚠".yellow());
        }
    }

    // Deallocate ports
    println!("  Deallocating ports...");
    match ports::deallocate(&worktree_state.allocation_key) {
        Ok(Some(ports)) => {
            println!(
                "  {} Deallocated ports {}-{}",
                "✓".green(),
                ports.first().unwrap_or(&0),
                ports.last().unwrap_or(&0)
            );
        }
        Ok(None) => {
            println!("  {} No ports were allocated", "⚠".yellow());
        }
        Err(e) => {
            println!("  {} Failed to deallocate ports: {}", "⚠".yellow(), e);
        }
    }

    // Change to home directory before removing worktree
    let home = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("/"));
    std::env::set_current_dir(&home)?;

    // Remove git worktree
    println!("  Removing git worktree...");
    match git::remove_worktree(&worktree_state.original_dir, &worktree_state.worktree_dir, true) {
        Ok(_) => {
            println!("  {} Git worktree removed", "✓".green());
        }
        Err(e) => {
            println!("  {} Failed to remove worktree: {}", "⚠".yellow(), e);
            // Try manual removal
            if let Err(e2) = std::fs::remove_dir_all(&worktree_state.worktree_dir) {
                println!("  {} Manual removal also failed: {}", "✗".red(), e2);
            } else {
                println!("  {} Manually removed directory", "✓".green());
            }
        }
    }

    println!();
    println!("{}", "Worktree closed successfully!".green().bold());
    println!();
    println!(
        "Return to original project:\n  cd {}",
        worktree_state.original_dir.display()
    );

    Ok(())
}

/// Resolve which worktree to close based on arguments
fn resolve_worktree(name: Option<String>, interactive: bool) -> Result<WorktreeState> {
    // If interactive mode, show selection
    if interactive {
        let worktrees = find_all_worktrees()?;
        if worktrees.is_empty() {
            bail!("No worktrees found.");
        }
        return select_worktree(&worktrees);
    }

    // If name provided, find by name
    if let Some(name) = name {
        let worktrees = find_all_worktrees()?;
        let matches: Vec<_> = worktrees
            .into_iter()
            .filter(|wt| wt.name == name || wt.allocation_key.ends_with(&format!("/{}", name)))
            .collect();

        match matches.len() {
            0 => bail!("No worktree found with name '{}'", name),
            1 => return Ok(matches.into_iter().next().unwrap()),
            _ => {
                println!("{}", "Multiple worktrees match that name:".yellow());
                return select_worktree(&matches);
            }
        }
    }

    // Try to detect current worktree
    if let Some(state) = crate::config::state::detect_worktree()? {
        return Ok(state);
    }

    // Not in a worktree - show interactive selection
    let worktrees = find_all_worktrees()?;
    if worktrees.is_empty() {
        bail!("No worktrees found.");
    }
    select_worktree(&worktrees)
}

/// Find all worktrees in the global directory
fn find_all_worktrees() -> Result<Vec<WorktreeState>> {
    let mut worktrees = Vec::new();
    let base_dir = paths::global_worktrees_dir();

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

/// Interactive worktree selection
fn select_worktree(worktrees: &[WorktreeState]) -> Result<WorktreeState> {
    println!("\n{}", "Select worktree to close:".bold());

    for (i, wt) in worktrees.iter().enumerate() {
        let port_range = if wt.ports.is_empty() {
            "no ports".to_string()
        } else {
            format!("{}-{}", wt.ports.first().unwrap(), wt.ports.last().unwrap())
        };

        println!(
            "  {}) {}/{} {} {}",
            (i + 1).to_string().cyan(),
            wt.project_name.blue(),
            wt.name.green(),
            format!("(ports {})", port_range).dimmed(),
            format!("[{}]", wt.branch).dimmed()
        );
    }

    print!("\n{} ", "Enter number:".bold());
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim();

    if input.is_empty() {
        bail!("No selection made.");
    }

    let idx: usize = input
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid number: {}", input))?;

    if idx == 0 || idx > worktrees.len() {
        bail!("Invalid selection: {}. Choose 1-{}", idx, worktrees.len());
    }

    Ok(worktrees[idx - 1].clone())
}
