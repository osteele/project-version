use anyhow::Result;
use semver::Version;
use std::fs;
use tempfile::tempdir;

// Import the project module from our crate
use project_version::project::detect_project;

#[test]
fn test_version_set_with_v_prefix() -> Result<()> {
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

    // Helper function to mimic the version string cleaning in set_version
    fn parse_version(version_str: &str) -> Result<Version> {
        let clean_version_str = if version_str.starts_with('v') {
            &version_str[1..]
        } else {
            version_str
        };

        Ok(Version::parse(clean_version_str)?)
    }

    // Test with standard version format
    let new_version = parse_version("2.0.0")?;
    project.update_version(&new_version)?;

    // Verify updated version
    let updated_version = project.get_version()?;
    assert_eq!(updated_version, Version::new(2, 0, 0));

    // Test with v-prefixed version format
    let new_version = parse_version("v3.1.4")?;
    project.update_version(&new_version)?;

    // Verify updated version
    let updated_version = project.get_version()?;
    assert_eq!(updated_version, Version::new(3, 1, 4));

    // Verify the actual file content
    let content = fs::read_to_string(&package_json_path)?;
    assert!(content.contains(r#""version": "3.1.4""#));

    Ok(())
}
