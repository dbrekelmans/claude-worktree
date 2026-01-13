# Claude Worktree Command

A Claude Code command for managing isolated git worktrees with automatic port allocation and lifecycle scripts.

## Features

- Create isolated git worktrees for parallel development
- Automatic port allocation to avoid conflicts between worktrees
- Lifecycle scripts (setup, run, stop, close) for each project
- Per-project configuration with team-shared and personal settings
- Automatic terminal launching for new worktrees

## Installation

Copy the files to your Claude configuration directory:

```bash
cp -r commands ~/.claude/
cp -r scripts ~/.claude/
```

## Usage

Use `/worktree` in Claude Code to manage worktrees.

### Commands

| Command | Description |
|---------|-------------|
| `/worktree` | Create a new worktree with allocated ports |
| `/worktree <param>` | Create worktree with parameter (configurable via SETUP.md) |
| `/worktree init` | Initialize worktree configuration for your project |
| `/worktree run` | Start the development environment |
| `/worktree stop` | Stop running services |
| `/worktree close` | Clean up and delete the worktree |
| `/worktree list` | Show all active worktrees |

### Getting Started

1. **Initialize your project** (first time only):
   ```
   /worktree init
   ```
   This creates configuration files and optionally generates lifecycle scripts based on your project.

2. **Create a worktree**:
   ```
   /worktree
   ```
   Or with a parameter (e.g., Linear issue ID):
   ```
   /worktree CHR-123
   ```

3. **In the worktree**, run services:
   ```
   /worktree run
   ```

4. **When done**, close the worktree:
   ```
   /worktree close
   ```

## Documentation

For detailed documentation on configuration, lifecycle scripts, port allocation, and more, see:

- [`scripts/worktree/README.template.md`](scripts/worktree/README.template.md) - Full documentation (also copied to your project during init)

## License

MIT License - see [LICENSE](LICENSE) for details.
