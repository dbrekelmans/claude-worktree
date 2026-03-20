use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};

use super::common;

pub fn execute_get(key: &str, worktree: Option<String>, file: &str) -> Result<()> {
    let worktree_dir = resolve_worktree_dir(worktree)?;
    let env_path = resolve_env_path(&worktree_dir, file);

    if !env_path.exists() {
        bail!("File not found: {}", env_path.display());
    }

    let contents = fs::read_to_string(&env_path)?;
    match find_value(&contents, key) {
        Some(value) => {
            println!("{}", value);
            Ok(())
        }
        None => bail!("Key '{}' not found in {}", key, env_path.display()),
    }
}

pub fn execute_set(
    key: &str,
    value: Option<String>,
    worktree: Option<String>,
    file: &str,
) -> Result<()> {
    let (actual_key, actual_value) = if let Some(val) = value {
        (key.to_string(), val)
    } else if let Some(pos) = key.find('=') {
        (key[..pos].to_string(), key[pos + 1..].to_string())
    } else {
        bail!("No value provided. Use: dotenv set KEY VALUE or dotenv set KEY=VALUE");
    };

    let worktree_dir = resolve_worktree_dir(worktree)?;
    let env_path = resolve_env_path(&worktree_dir, file);

    if env_path.exists() {
        let contents = fs::read_to_string(&env_path)?;
        let updated = set_value(&contents, &actual_key, &actual_value);
        fs::write(&env_path, updated)?;
    } else {
        if let Some(parent) = env_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&env_path, format!("{}={}\n", actual_key, actual_value))?;
    }

    Ok(())
}

fn resolve_env_path(worktree_dir: &Path, file: &str) -> PathBuf {
    let file_path = PathBuf::from(file);
    if file_path.is_absolute() {
        file_path
    } else {
        worktree_dir.join(file)
    }
}

fn resolve_worktree_dir(name: Option<String>) -> Result<PathBuf> {
    match common::resolve_worktree(name)? {
        Some(state) => Ok(state.worktree_dir),
        None => bail!("Not in a worktree directory. Use --worktree to specify one."),
    }
}

/// Find the value for a key in .env file contents.
/// Strips surrounding quotes (single or double) from the value.
fn find_value(contents: &str, key: &str) -> Option<String> {
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }
        if let Some(pos) = trimmed.find('=') {
            let k = trimmed[..pos].trim();
            if k == key {
                let v = trimmed[pos + 1..].trim();
                return Some(strip_quotes(v).to_string());
            }
        }
    }
    None
}

/// Update an existing key or append a new one. Preserves comments and blank lines.
fn set_value(contents: &str, key: &str, value: &str) -> String {
    let mut result = String::new();
    let mut found = false;

    for line in contents.lines() {
        let trimmed = line.trim();
        if !found && !trimmed.starts_with('#') && !trimmed.is_empty() {
            if let Some(pos) = trimmed.find('=') {
                let k = trimmed[..pos].trim();
                if k == key {
                    result.push_str(&format!("{}={}", key, value));
                    result.push('\n');
                    found = true;
                    continue;
                }
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    if !found {
        // If file didn't end with newline and has content, add one
        if !contents.is_empty() && !contents.ends_with('\n') {
            result.push('\n');
        }
        result.push_str(&format!("{}={}\n", key, value));
    }

    result
}

fn strip_quotes(s: &str) -> &str {
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_value_simple() {
        let contents = "PORT=3000\nHOST=localhost\n";
        assert_eq!(find_value(contents, "PORT"), Some("3000".to_string()));
        assert_eq!(find_value(contents, "HOST"), Some("localhost".to_string()));
        assert_eq!(find_value(contents, "MISSING"), None);
    }

    #[test]
    fn test_find_value_quoted() {
        let contents = "NAME=\"hello world\"\nOTHER='single'\n";
        assert_eq!(
            find_value(contents, "NAME"),
            Some("hello world".to_string())
        );
        assert_eq!(find_value(contents, "OTHER"), Some("single".to_string()));
    }

    #[test]
    fn test_find_value_with_comments() {
        let contents = "# A comment\nPORT=3000\n# Another\nHOST=localhost\n";
        assert_eq!(find_value(contents, "PORT"), Some("3000".to_string()));
        assert_eq!(find_value(contents, "HOST"), Some("localhost".to_string()));
    }

    #[test]
    fn test_set_value_update_existing() {
        let contents = "PORT=3000\nHOST=localhost\n";
        let result = set_value(contents, "PORT", "4000");
        assert_eq!(result, "PORT=4000\nHOST=localhost\n");
    }

    #[test]
    fn test_set_value_append_new() {
        let contents = "PORT=3000\n";
        let result = set_value(contents, "HOST", "localhost");
        assert_eq!(result, "PORT=3000\nHOST=localhost\n");
    }

    #[test]
    fn test_set_value_preserves_comments() {
        let contents = "# Database config\nDB_HOST=localhost\n\n# App config\nPORT=3000\n";
        let result = set_value(contents, "PORT", "4000");
        assert_eq!(
            result,
            "# Database config\nDB_HOST=localhost\n\n# App config\nPORT=4000\n"
        );
    }

    #[test]
    fn test_set_value_empty_file() {
        let contents = "";
        let result = set_value(contents, "PORT", "3000");
        assert_eq!(result, "PORT=3000\n");
    }

    #[test]
    fn test_strip_quotes() {
        assert_eq!(strip_quotes("\"hello\""), "hello");
        assert_eq!(strip_quotes("'hello'"), "hello");
        assert_eq!(strip_quotes("hello"), "hello");
        assert_eq!(strip_quotes("\"\""), "");
    }
}
