# project-version

[![CI](https://github.com/osteele/project-version/actions/workflows/ci.yml/badge.svg)](https://github.com/osteele/project-version/actions/workflows/ci.yml)
[![Docs](https://github.com/osteele/project-version/actions/workflows/docs.yml/badge.svg)](https://osteele.github.io/project-version/)
[![Crate](https://img.shields.io/crates/v/project-version.svg)](https://crates.io/crates/project-version)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)

A cross-language project version bumper CLI that supports multiple project types.

## Features

- Bump the version number in project files (major, minor, or patch)
- Set a specific version number directly
- Support for multiple project types:
  - Node.js (package.json)
  - Python (pyproject.toml)
  - Rust (Cargo.toml)
  - Go (version.go files)
  - Ruby (Gemfile, gemspec, version.rb)
- Automatically update CHANGELOG files
- Automatically update lock files with appropriate package managers
  - npm, yarn, pnpm, bun for Node.js
  - uv, poetry, pipenv, pdm for Python
  - cargo for Rust
  - bundle for Ruby
  - go mod for Go
- Git integration - commit changes and tag releases
- Dry-run mode for safer execution

## Installation

### From crates.io

```bash
cargo install project-version
```

### From Source

```bash
git clone https://github.com/osteele/project-version.git
cd project-version
cargo install --path .
```

## Usage

```
project-version [OPTIONS] [DIRECTORY] [COMMAND]
```

### Commands:
- `bump` - Bump project version (major, minor, or patch)
- `set` - Set project version to a specific version number
- `help` - Print help information

### Arguments:
- `[DIRECTORY]` - Project directory to bump (defaults to current directory)

### Options:
- `-v, --verbose` - Verbose output
- `-n, --dry-run` - Dry run (no file modifications or git operations)
- `-h, --help` - Print help
- `-V, --version` - Print version

### Bump Command Options:
- `[BUMP_TYPE]` - Type of version bump to perform: major, minor, or patch (default: patch)
- `--no-commit` - Skip committing changes
- `--no-tag` - Skip tagging the commit
- `--no-lockupdate` - Skip updating lock files (package-lock.json, Cargo.lock, etc.)
- `--force-tag` - Force tag creation (overwrite existing tag)

### Set Command Options:
- `<VERSION>` - Version number to set (must be a valid semver string, can include optional 'v' prefix)
- `--no-commit` - Skip committing changes
- `--no-tag` - Skip tagging the commit
- `--force-tag` - Force tag creation (overwrite existing tag)
- `--no-lockupdate` - Skip updating lock files (package-lock.json, Cargo.lock, etc.)
- `--force` - Force setting version even if it's lower than current version

## Examples

```bash
# Show current version and available commands
project-version

# Bump patch version in current directory
project-version bump

# Bump minor version
project-version bump minor

# Bump major version with verbose output
project-version bump major --verbose

# Set a specific version (with or without v prefix)
project-version set 2.0.0
project-version set v2.0.0

# Set a lower version (requires --force)
project-version set 1.0.0 --force

# Dry run to see what would happen
project-version bump --dry-run
project-version set 2.0.0 --dry-run

# Bump version without creating a git commit
project-version bump --no-commit

# Bump version in a specific directory
project-version /path/to/project bump
```

## Development Setup

This project uses [just](https://github.com/casey/just) as a command runner for development tasks.

### Quick Start

1. Install dependencies:
   ```bash
   cargo build
   ```

2. Install the pre-commit tool (if not already installed):
   ```bash
   pip install pre-commit
   ```

3. Set up the development environment:
   ```bash
   just setup
   ```

For more detailed development instructions, see [DEVELOPMENT.md](docs/DEVELOPMENT.md).

## Supported Project Files

- **Node.js**: Updates the version field in package.json
  - Detects and runs npm, yarn, pnpm, or bun to update dependencies
- **Python**: Updates the version in pyproject.toml
  - Detects and runs uv, poetry, pipenv, pdm, or pip to update dependencies
- **Rust**: Updates the version in Cargo.toml
  - Runs cargo update to update dependencies
- **Go**: Updates version strings in version.go files
  - Runs go mod tidy to update dependencies
- **Ruby**: Updates versions in gemspec and version.rb files
  - Runs bundle install to update dependencies
- **Helm charts** Updates version in the Chart.yaml file

## Acknowledgements

project-version is built with:
- [clap](https://github.com/clap-rs/clap) - Command line argument parsing
- [semver](https://github.com/dtolnay/semver) - Semantic versioning
- [anyhow](https://github.com/dtolnay/anyhow) - Error handling
- [regex](https://github.com/rust-lang/regex) - Regular expressions
- [toml](https://github.com/alexcrichton/toml-rs) - TOML parsing
- [serde](https://github.com/serde-rs/serde) - Serialization framework
- [serde_json](https://github.com/serde-rs/json) - JSON handling
- [chrono](https://github.com/chronotope/chrono) - Date and time handling
- [colored](https://github.com/mackwic/colored) - Terminal colors
- [dialoguer](https://github.com/console-rs/dialoguer) - User prompts

## Contributors

- [Oliver Steele](https://github.com/osteele) - Project maintainer
- [Marco Fuykschot](https://github.com/MarcoFuykschot) - Rust workspace package version support, `--no-lockupdate` flag

## License

MIT [Oliver Steele](https://github.com/osteele)