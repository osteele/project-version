# project-version

A cross-language project version bumper CLI that supports multiple project types.

## Features

- Bump the version number in project files (major, minor, or patch)
- Support for multiple project types:
  - Node.js (package.json)
  - Python (pyproject.toml)
  - Rust (Cargo.toml)
  - Go (version.go files)
- Automatically update CHANGELOG files
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
project-version [OPTIONS] [DIRECTORY]
```

### Arguments:
- `[DIRECTORY]` - Project directory to bump (defaults to current directory)

### Options:
- `major|minor|patch` - Type of version bump to perform (default: patch)
- `--no-commit` - Skip committing changes
- `--no-tag` - Skip tagging the commit
- `--force-tag` - Force tag creation (overwrite existing tag)
- `-v, --verbose` - Verbose output
- `-n, --dry-run` - Dry run (no file modifications or git operations)
- `-h, --help` - Print help
- `-V, --version` - Print version

## Examples

```bash
# Bump patch version in current directory
project-version

# Bump minor version
project-version minor

# Bump major version with verbose output
project-version major --verbose

# Dry run to see what would happen
project-version --dry-run

# Bump version without creating a git commit
project-version --no-commit

# Bump version in a specific directory
project-version /path/to/project
```

## Supported Project Files

- **Node.js**: Updates the version field in package.json
- **Python**: Updates the version in pyproject.toml
- **Rust**: Updates the version in Cargo.toml
- **Go**: Updates version strings in version.go files
- **Ruby**: Updates versions in gemspec and version.rb files

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

## License

MIT © [Oliver Steele](https://github.com/osteele)