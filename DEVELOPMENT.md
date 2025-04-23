# Development Guide

This document provides information for contributors who want to understand and work on `project-version`.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [just](https://github.com/casey/just) - Command runner

## Initial Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/osteele/project-version.git
   cd project-version
   ```

2. Set up the development environment:
   ```bash
   just setup
   ```
   This will build the project and set up Git hooks via cargo-husky.

## Development Workflow

### Common Tasks

The project uses `just` as a command runner. Here are the available commands:

- `just format` - Format code with cargo fmt
- `just lint` - Run clippy linter with warnings as errors
- `just typecheck` - Run type checking
- `just test` - Run tests
- `just check` - Run all checks (format, typecheck, lint, and test)
- `just fix` - Automatically fix linting issues where possible
- `just build` - Build the project
- `just run [ARGS]` - Run the project with arguments
- `just clean` - Clean build artifacts

To see all available commands:
```bash
just
```

### Git Hooks

The project uses cargo-husky to manage Git hooks. These hooks run automatically when you commit changes and will prevent commits if any checks fail.

The following hooks are configured:
- Code formatting (rustfmt)
- Linting with clippy
- Running tests

If you need to bypass the hooks temporarily, you can use the `--no-verify` flag with git commit:
```bash
git commit --no-verify
```

## Project Structure

- `src/main.rs` - Entry point and CLI argument parsing
- `src/project.rs` - Project type detection and version handling
- `src/changelog.rs` - CHANGELOG file detection and updating
- `src/git.rs` - Git operations (commit changes and create tags)

## Adding Support for a New Project Type

To add support for a new project type:

1. Add a new struct in `src/project.rs` that implements the `Project` trait
2. Update the `detect_project` function to check for the new project type
3. Implement the required methods:
   - `get_version` - Extract the current version
   - `update_version` - Update the version in project files
   - `get_file_path` - Return the path to the main project file
   - `get_files_to_commit` - Return the paths to all files that should be committed

## Building for Release

```bash
cargo build --release
```

The binary will be available in `target/release/project-version`.

## Release Process

1. Ensure all tests pass with `just check`
2. Update the CHANGELOG.md file
3. Use the tool itself to bump the version:
   ```bash
   cargo run -- bump minor
   ```
   Or for a specific version:
   ```bash
   cargo run -- set 1.2.3
   ```
4. Push changes and tags to GitHub
5. Create a new GitHub release
6. Publish to crates.io:
   ```bash
   cargo publish
   ```

## Code Style and Guidelines

- Run `cargo fmt` before committing to ensure consistent formatting
- Run `cargo clippy` to catch common mistakes and improve code quality
- Write tests for new functionality
- Update documentation when adding new features

## Dependencies

Main dependencies and their purposes:

- `clap` - Command-line argument parsing
- `semver` - Semantic version parsing and manipulation
- `anyhow` - Error handling
- `regex` - Pattern matching for version updates
- `toml` - TOML file parsing
- `serde` - Serialization/deserialization
- `serde_json` - JSON handling
- `log` and `env_logger` - Logging
- `chrono` - Date and time handling
- `colored` - Colorized terminal output
- `dialoguer` - Interactive user prompts
- `cargo-husky` - Git hooks management