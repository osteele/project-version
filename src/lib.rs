//! # project-version
//!
//! [![CI](https://github.com/osteele/project-version/actions/workflows/ci.yml/badge.svg)](https://github.com/osteele/project-version/actions/workflows/ci.yml)
//! [![Docs](https://github.com/osteele/project-version/actions/workflows/docs.yml/badge.svg)](https://osteele.github.io/project-version/)
//! [![Crate](https://img.shields.io/crates/v/project-version.svg)](https://crates.io/crates/project-version)
//! [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
//!
//! A cross-language project version bumper CLI that supports multiple project types.
//!
//! ## Supported Project Types
//!
//! - Node.js (package.json)
//! - Python (pyproject.toml)
//! - Rust (Cargo.toml)
//! - Go (version.go files)
//! - Ruby (Gemfile, gemspec, version.rb)
//!
//! ## Usage
//!
//! ```bash
//! # Show current version and available commands
//! project-version
//!
//! # Bump patch version in current directory
//! project-version bump
//!
//! # Bump minor version
//! project-version bump minor
//!
//! # Bump major version with verbose output
//! project-version bump major --verbose
//!
//! # Set a specific version (with or without v prefix)
//! project-version set 2.0.0
//! project-version set v2.0.0
//!
//! # Set a lower version (requires --force)
//! project-version set 1.0.0 --force
//!
//! # Dry run to see what would happen
//! project-version bump --dry-run
//! project-version set 2.0.0 --dry-run
//!
//! # Bump version without creating a git commit
//! project-version bump --no-commit
//!
//! # Bump version in a specific directory
//! project-version /path/to/project bump
//! ```
//!
//! ## Options
//!
//! - `--dry-run` - Show what would happen without making changes
//! - `--verbose` - Show more detailed output
//! - `--no-commit` - Don't create a git commit
//! - `--force` - Force setting version even if it's lower than current version

pub mod changelog;
pub mod git;
pub mod project;
