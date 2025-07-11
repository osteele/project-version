# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Support for Rust workspace package versions ([#1](https://github.com/osteele/project-version/pull/1)) - Thanks [@MarcoFuykschot](https://github.com/MarcoFuykschot)!

### Added

- CI/CD improvements:
  - Added GitHub Actions workflow for CI (lint, typecheck, test)
  - Added GitHub Actions workflow for building and publishing documentation
  - Added badges to README for CI status, documentation, and crates.io
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
