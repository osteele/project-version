use anyhow::Result;
use semver::Version;
use std::fs;
use tempfile::tempdir;

// Import the project module from our crate
use project_version::project::detect_project;

#[test]
fn test_node_project_detection() -> Result<()> {
    let temp_dir = tempdir()?;
    let package_json_path = temp_dir.path().join("package.json");

    // Create a minimal package.json
    fs::write(
        &package_json_path,
        r#"{
  "name": "test-project",
  "version": "1.2.3",
  "description": "Test project"
}
"#,
    )?;

    // Detect project type
    let project = detect_project(temp_dir.path().to_str().unwrap())?;

    // Verify detected version
    let version = project.get_version()?;
    assert_eq!(version, Version::new(1, 2, 3));

    // Test update
    let new_version = Version::new(2, 0, 0);
    project.update_version(&new_version)?;

    // Verify updated version
    let updated_content = fs::read_to_string(&package_json_path)?;
    assert!(updated_content.contains(r#""version": "2.0.0""#));

    Ok(())
}

#[test]
fn test_python_project_detection() -> Result<()> {
    let temp_dir = tempdir()?;
    let pyproject_path = temp_dir.path().join("pyproject.toml");

    // Create a minimal pyproject.toml
    fs::write(
        &pyproject_path,
        r#"[project]
name = "test-project"
version = "0.5.1"
description = "Test project"
"#,
    )?;

    // Detect project type
    let project = detect_project(temp_dir.path().to_str().unwrap())?;

    // Verify detected version
    let version = project.get_version()?;
    assert_eq!(version, Version::new(0, 5, 1));

    // Test update
    let new_version = Version::new(0, 6, 0);
    project.update_version(&new_version)?;

    // Verify updated version
    let updated_content = fs::read_to_string(&pyproject_path)?;
    assert!(updated_content.contains(r#"version = "0.6.0""#));

    Ok(())
}

#[test]
fn test_rust_project_detection() -> Result<()> {
    let temp_dir = tempdir()?;
    let cargo_path = temp_dir.path().join("Cargo.toml");

    // Create a minimal Cargo.toml
    fs::write(
        &cargo_path,
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )?;

    // Detect project type
    let project = detect_project(temp_dir.path().to_str().unwrap())?;

    // Verify detected version
    let version = project.get_version()?;
    assert_eq!(version, Version::new(0, 1, 0));

    // Test update
    let new_version = Version::new(0, 1, 1);
    project.update_version(&new_version)?;

    // Verify updated version
    let updated_content = fs::read_to_string(&cargo_path)?;
    assert!(updated_content.contains(r#"version = "0.1.1""#));

    Ok(())
}

#[test]
fn test_rust_workspace_detection() -> Result<()> {
    let temp_dir = tempdir()?;
    let cargo_path = temp_dir.path().join("Cargo.toml");

    // Create a minimal Cargo.toml
    fs::write(
        &cargo_path,
        r#"[workspace]
name = "test-project"

[workspace.package]
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
    )?;

    // Detect project type
    let project = detect_project(temp_dir.path().to_str().unwrap())?;

    // Verify detected version
    let version = project.get_version()?;
    assert_eq!(version, Version::new(0, 1, 0));

    // Test update
    let new_version = Version::new(0, 1, 1);
    project.update_version(&new_version)?;

    // Verify updated version
    let updated_content = fs::read_to_string(&cargo_path)?;
    assert!(updated_content.contains(r#"version = "0.1.1""#));

    Ok(())
}

#[test]
fn test_changelog_update() -> Result<()> {
    let temp_dir = tempdir()?;
    let changelog_path = temp_dir.path().join("CHANGELOG.md");

    // Create a simple CHANGELOG.md with an unreleased section - make sure line endings match what we expect
    fs::write(
        &changelog_path,
        "# Changelog\n\n## [Unreleased]\n\n### Added\n- Feature 1\n- Feature 2\n\n## [0.1.0] - 2023-01-01\n\n### Added\n- Initial release\n"
    )?;

    // Update the changelog
    let new_version = Version::new(0, 2, 0);
    project_version::changelog::update_changelog(&changelog_path, &new_version)?;

    // Verify updated changelog
    let updated_content = fs::read_to_string(&changelog_path)?;

    // Check that the changelog was properly updated

    // Just check that Unreleased was replaced with the version number
    assert!(updated_content.contains("[0.2.0]"));
    assert!(!updated_content.contains("[Unreleased]"));

    Ok(())
}
