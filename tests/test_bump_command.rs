use anyhow::Result;
use semver::Version;
use std::fs;
use tempfile::tempdir;

// Import the project module from our crate
use project_version::project::detect_project;

#[test]
fn test_bump_command_functionality() -> Result<()> {
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

    // Verify initial version
    let version = project.get_version()?;
    assert_eq!(version, Version::new(1, 2, 3));

    // Test patch bump
    let new_version = Version::new(1, 2, 4); // Patch bump from 1.2.3
    project.update_version(&new_version)?;

    // Verify patch bump
    let updated_version = project.get_version()?;
    assert_eq!(updated_version, Version::new(1, 2, 4));

    // Test minor bump
    let new_version = Version::new(1, 3, 0); // Minor bump from 1.2.4
    project.update_version(&new_version)?;

    // Verify minor bump
    let updated_version = project.get_version()?;
    assert_eq!(updated_version, Version::new(1, 3, 0));

    // Test major bump
    let new_version = Version::new(2, 0, 0); // Major bump from 1.3.0
    project.update_version(&new_version)?;

    // Verify major bump
    let updated_version = project.get_version()?;
    assert_eq!(updated_version, Version::new(2, 0, 0));

    // Verify the actual file content
    let content = fs::read_to_string(&package_json_path)?;
    assert!(content.contains(r#""version": "2.0.0""#));

    Ok(())
}
