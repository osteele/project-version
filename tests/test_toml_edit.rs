use std::fs;
use tempfile::tempdir;
use anyhow::Result;

#[test]
fn test_toml_edit_preserves_comments_and_formatting() -> Result<()> {
    let temp_dir = tempdir()?;
    let cargo_toml_path = temp_dir.path().join("Cargo.toml");
    
    // Create a sample Cargo.toml with comments and custom formatting
    fs::write(
        &cargo_toml_path,
        r#"# This is a header comment
[package]
name = "test-project"  # Project name
# Version comment
version = "0.1.0"
edition = "2021"

# Dependencies section
[dependencies]
# Important dependency
some-lib = "1.0"  # With comment 
"#,
    )?;
    
    // Parse with toml_edit
    let content = fs::read_to_string(&cargo_toml_path)?;
    let mut doc = content.parse::<toml_edit::DocumentMut>()?;
    
    // Update the version
    if let Some(package) = doc.get_mut("package") {
        if let Some(package_table) = package.as_table_mut() {
            if package_table.contains_key("version") {
                package_table["version"] = toml_edit::value("0.2.0");
            }
        }
    }
    
    // Write the updated TOML
    let new_content = doc.to_string();
    fs::write(&cargo_toml_path, &new_content)?;
    
    // Read back and verify
    let updated_content = fs::read_to_string(&cargo_toml_path)?;
    
    // Check that comments and formatting are preserved
    assert!(updated_content.contains("# This is a header comment"));
    assert!(updated_content.contains("name = \"test-project\"  # Project name"));
    assert!(updated_content.contains("# Version comment"));
    assert!(updated_content.contains("version = \"0.2.0\""));
    assert!(updated_content.contains("# Important dependency"));
    assert!(updated_content.contains("some-lib = \"1.0\"  # With comment"));
    
    Ok(())
}