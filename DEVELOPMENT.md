# Development Guide

This document provides information for contributors who want to understand and work on `polybump`.

## Project Structure

- `src/main.rs` - Entry point and CLI argument parsing
- `src/project.rs` - Project type detection and version handling
- `src/changelog.rs` - CHANGELOG file detection and updating
- `src/git.rs` - Git operations (commit changes and create tags)

## Development Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/osteele/polybump.git
   cd polybump
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run tests:
   ```bash
   cargo test
   ```

4. Run the CLI in development:
   ```bash
   cargo run -- [arguments]
   ```

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

The binary will be available in `target/release/polybump`.

## Code Style and Guidelines

- Run `cargo fmt` before committing to ensure consistent formatting
- Run `cargo clippy` to catch common mistakes and improve code quality
- Write tests for new functionality
- Update documentation when adding new features

## Release Process

1. Update the version in `Cargo.toml`
2. Update the CHANGELOG.md
3. Commit the changes
4. Create a new tag matching the version
5. Push the changes and tag to GitHub
6. Publish to crates.io:
   ```bash
   cargo publish
   ```

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