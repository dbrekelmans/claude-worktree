# CLAUDE.md

This file provides guidance to Claude Code when working with code in this repository.

## Overview

This is a Rust CLI tool (`worktree`) for managing isolated git worktrees with automatic port allocation and lifecycle scripts.

## Build Commands

```bash
cargo build              # Debug build
cargo build --release    # Release build
cargo check              # Check for errors without building
cargo test               # Run tests
cargo clippy             # Run linter
```

## Architecture

### Project Structure

```
src/
├── main.rs              # Entry point, CLI routing
├── cli.rs               # Clap CLI definitions
├── names.rs             # Random name generation
├── commands/            # Command implementations
│   ├── init.rs          # worktree init
│   ├── new.rs           # worktree [new] [param]
│   ├── run.rs           # worktree run
│   ├── stop.rs          # worktree stop
│   ├── close.rs         # worktree close
│   └── list.rs          # worktree list
├── config/              # Configuration management
│   ├── paths.rs         # Path utilities
│   ├── settings.rs      # Settings structs
│   └── state.rs         # Worktree state
├── ports/               # Port allocation
│   ├── allocator.rs     # Allocation logic
│   └── checker.rs       # Port availability
├── git/                 # Git operations
│   └── worktree.rs      # Git worktree commands
├── terminal/            # Terminal integration
│   └── launcher.rs      # Terminal detection/launch
└── scripts/             # Script handling
    ├── runner.rs        # Script execution
    └── generator.rs     # Claude CLI integration
```

### Key Data Files

- `~/.worktree/port-allocations.json` - Global port tracking
- `.worktree/settings.json` - Per-project team settings
- `.worktree/settings.local.json` - Per-project personal settings
- `state.json` - Per-worktree state (in worktree root)

### Configuration Schema

**settings.json** (team-shared):
```json
{
  "portCount": 10,
  "portRangeStart": 50000,
  "portRangeEnd": 60000,
  "branchPrefix": "worktree/",
  "autoLaunchTerminal": true
}
```

**state.json** (per-worktree):
```json
{
  "name": "swift-falcon-a3b2",
  "projectName": "my-project",
  "originalDir": "/path/to/project",
  "worktreeDir": "/path/to/worktree",
  "branch": "worktree/swift-falcon-a3b2",
  "ports": [50000, 50001, ...],
  "allocationKey": "my-project/swift-falcon-a3b2",
  "createdAt": "2026-01-13T10:00:00Z"
}
```

## Testing

### Structure

```
tests/
├── helpers/
│   └── mod.rs           # TestEnv shared helper
├── general.rs           # help, version, no-args
├── init.rs              # init command tests
├── new.rs               # new command tests
├── list.rs              # list command tests
├── status.rs            # status command tests
├── path.rs              # path command tests
├── close.rs             # close command tests
├── rename.rs            # rename command tests
├── dotenv.rs            # dotenv command tests
├── run_stop.rs          # run and stop command tests
├── cp.rs                # cp command tests
└── completions.rs       # completions command test
```

### Running tests

```bash
cargo test               # All tests (unit + integration)
cargo test --test init   # Run only init tests
cargo test --test new    # Run only new tests
```

### TestEnv helper

All integration tests use `TestEnv` from `tests/helpers/mod.rs`, which creates a fully isolated environment:
- Fake `$HOME` in a temp directory (cleaned up on drop)
- Pre-seeded `~/.config/worktree/config.json` to skip interactive setup
- A git repo at `$HOME/project` with one commit

Key helper methods:
- `cmd()` / `cmd_in(dir)` — build a `Command` targeting the binary with the fake HOME
- `init_project()` — runs `init --defaults --no-scripts --no-ai`
- `init_project_with_scripts()` — runs `init --defaults --no-ai` (generates template scripts)
- `create_worktree()` / `create_worktree_with_scripts()` — init + new, returns the worktree name
- `find_worktree_dir(name)` — resolves `~/.worktree/worktrees/project/<name>`
- `git(args)` — run git commands in the test repo

### Writing new tests

- One test file per command (or closely related pair like run/stop)
- Each file declares `mod helpers;` and uses `helpers::TestEnv`
- Tests verify both stdout/stderr output AND filesystem side-effects (files created, JSON content, permissions)
- Use `--no-ai` flag on init to avoid Claude CLI calls in tests
- For tests that need scripts in worktrees: use `create_worktree_with_scripts()` which commits `.worktree/` to git before creating the worktree (since `git worktree add` only copies tracked files)
- Commands not tested: `open` (requires terminal emulator), `cleanup` (requires interactive selection)

## Development Notes

- Issue tracking: https://linear.app/riotbyte/team/TREE
- Uses `clap` for CLI parsing with derive macros
- Uses `serde` for JSON serialization
- Uses `anyhow` for error handling
- Uses `socket2` for port availability checking
- Cross-platform terminal detection (macOS + Linux)
- Optional Claude CLI integration for script generation
- Never use #[allow(dead_code)] or #[allow(unused_variables)]
- After implementing new features of changes, make sure to run `cargo test`, `cargo clippy` and `cargo fmt`
