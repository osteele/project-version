# Justfile for project-version

# Default recipe to run when just is called without arguments
default:
    @just --list

# Build the project
build:
    cargo build

# Run all checks (lint, typecheck, and test)
check: lint typecheck test
    @echo "All checks passed!"

# Clean build artifacts
clean:
    cargo clean

# Fix linting issues automatically where possible
fix:
    cargo fix --allow-dirty --allow-staged
    cargo clippy --fix --allow-dirty --allow-staged

# Run cargo fmt to format code
format:
    cargo fmt

# Run cargo clippy to check for lints
lint:
    cargo fmt --check
    cargo clippy -- -D warnings

# Build with optimizations
release:
    cargo build --release

# Run the project
run *ARGS:
    cargo run -- {{ARGS}}

# Run type checking
typecheck:
    cargo check

# Run tests
test:
    cargo test

# Run tests with output
test-verbose:
    cargo test -- --nocapture
