use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::config::{paths, state::WorktreeState};

enum DirRole {
    Source,
    Destination,
}

pub fn execute(
    path: String,
    from: Option<String>,
    to: Option<String>,
    force: bool,
    create_if_not_exists: bool,
    skip_if_not_exists: bool,
    reverse: bool,
) -> Result<()> {
    let rel_path = Path::new(&path);
    if rel_path.is_absolute() {
        bail!("Path must be relative, got: {}", path);
    }

    let mut from_dir = resolve_dir(from, DirRole::Source)?;
    let mut to_dir = resolve_dir(to, DirRole::Destination)?;

    if reverse {
        std::mem::swap(&mut from_dir, &mut to_dir);
    }

    let from_canonical = fs::canonicalize(&from_dir).unwrap_or_else(|_| from_dir.clone());
    let to_canonical = fs::canonicalize(&to_dir).unwrap_or_else(|_| to_dir.clone());

    if from_canonical == to_canonical {
        bail!(
            "Source and destination resolve to the same directory: {}",
            from_canonical.display()
        );
    }

    let src_path = from_dir.join(rel_path);
    let dst_path = to_dir.join(rel_path);

    if !src_path.exists() {
        if skip_if_not_exists {
            return Ok(());
        }
        if create_if_not_exists {
            if let Some(parent) = dst_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&dst_path, "")?;
            println!("Created empty file: {}", dst_path.display());
            return Ok(());
        }
        bail!("Source does not exist: {}", src_path.display());
    }

    if dst_path.exists() && !force {
        bail!(
            "Destination already exists: {} (use --force to overwrite)",
            dst_path.display()
        );
    }

    if src_path.is_dir() {
        copy_dir_recursive(&src_path, &dst_path)?;
        println!(
            "Copied directory {} -> {}",
            src_path.display(),
            dst_path.display()
        );
    } else {
        if let Some(parent) = dst_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&src_path, &dst_path)?;
        println!("Copied {} -> {}", src_path.display(), dst_path.display());
    }

    Ok(())
}

fn resolve_dir(name: Option<String>, role: DirRole) -> Result<PathBuf> {
    if let Some(name) = name {
        let all_worktrees = find_all_worktrees()?;
        let matches: Vec<_> = all_worktrees
            .into_iter()
            .filter(|wt| wt.matches_identifier(&name))
            .collect();

        match matches.len() {
            0 => bail!("No worktree found with name '{}'", name),
            1 => return Ok(matches.into_iter().next().unwrap().worktree_dir),
            _ => bail!(
                "Multiple worktrees match '{}'. Please be more specific.",
                name
            ),
        }
    }

    match crate::config::state::detect_worktree()? {
        Some(state) => match role {
            DirRole::Source => Ok(state.original_dir),
            DirRole::Destination => Ok(state.worktree_dir),
        },
        None => bail!("Not in a worktree directory. Use --from/--to to specify directories."),
    }
}

fn find_all_worktrees() -> Result<Vec<WorktreeState>> {
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

    Ok(worktrees)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    for entry in WalkDir::new(src).into_iter().filter_map(|e| e.ok()) {
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_dir_recursive_creates_structure() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        // Create source structure
        fs::write(src_dir.path().join("file.txt"), "hello").unwrap();
        fs::create_dir_all(src_dir.path().join("sub")).unwrap();
        fs::write(src_dir.path().join("sub/nested.txt"), "world").unwrap();

        let dst = dst_dir.path().join("output");
        copy_dir_recursive(src_dir.path(), &dst).unwrap();

        assert_eq!(fs::read_to_string(dst.join("file.txt")).unwrap(), "hello");
        assert_eq!(
            fs::read_to_string(dst.join("sub/nested.txt")).unwrap(),
            "world"
        );
    }

    #[test]
    fn test_copy_dir_recursive_empty_dir() {
        let src_dir = TempDir::new().unwrap();
        let dst_dir = TempDir::new().unwrap();

        let dst = dst_dir.path().join("output");
        copy_dir_recursive(src_dir.path(), &dst).unwrap();

        assert!(dst.exists());
        assert!(dst.is_dir());
    }
}
