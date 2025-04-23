# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Development tooling improvements:
  - Added `justfile` with common development commands
  - Added cargo-husky configuration for Git hooks
  - Added documentation for development setup
- `set` command to set a specific version number directly
  - Accepts version numbers with or without 'v' prefix (e.g., `2.0.0` or `v2.0.0`)
  - Includes validation to prevent setting lower versions without `--force` flag
  - Supports dry-run mode
- Initial release of polybump
- Support for Node.js (package.json)
- Support for Python (pyproject.toml)
- Support for Rust (Cargo.toml)
- Support for Go (version.go files)
- CHANGELOG updating
- Git integration (commit and tag)