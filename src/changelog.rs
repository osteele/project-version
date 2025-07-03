use anyhow::{Context, Result};
use chrono::Local;
use log::{debug, warn};
use regex::Regex;
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};

// Regex patterns for finding changelog files (case-insensitive)
const CHANGELOG_PATTERNS: [&str; 3] = [
    r"(?i)^changelog(\.md)?$", // CHANGELOG.md, Changelog.md, changelog, etc.
    r"(?i)^changes(\.md)?$",   // CHANGES.md, Changes.md, changes, etc.
    r"(?i)^history(\.md)?$",   // HISTORY.md, History.md, etc.
];

// Regex patterns for unreleased section headers
// Using (?m) for multiline mode so ^ matches start of any line
const UNRELEASED_PATTERNS: [&str; 3] = [
    r"(?mi)^##\s*\[unreleased\]",            // ## [Unreleased]
    r"(?mi)^##\s+unreleased",                // ## Unreleased
    r"(?mi)^\[unreleased\](?:\s*)?(?:\n|$)", // [Unreleased] at start of line
];

/// Find a changelog file in the specified directory
pub fn find_changelog(dir: &str) -> Option<PathBuf> {
    let dir_path = Path::new(dir);

    if let Ok(entries) = fs::read_dir(dir_path) {
        for entry in entries.filter_map(Result::ok) {
            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            for pattern in &CHANGELOG_PATTERNS {
                let re = Regex::new(pattern).unwrap();
                if re.is_match(&file_name_str) {
                    let path = entry.path();
                    debug!("Found changelog: {}", path.display());
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Update the changelog by replacing the unreleased section with the new version
pub fn update_changelog(path: &Path, version: &Version) -> Result<()> {
    let content = fs::read_to_string(path).context("Failed to read changelog file")?;

    let today = Local::now().format("%Y-%m-%d").to_string();

    // Try to find and replace an unreleased section using regex
    let mut new_content = content.clone();
    let mut found = false;

    // Format the version header
    let version_header = format!("## [{version}] - {today}");

    // Try each pattern until one matches
    for pattern in &UNRELEASED_PATTERNS {
        // Compile the regex with proper flags
        let re = match Regex::new(pattern) {
            Ok(re) => re,
            Err(e) => {
                warn!("Invalid regex pattern: {pattern} - {e}");
                continue;
            }
        };

        // Check if this pattern matches
        if re.is_match(&content) {
            // It matched! Replace the first occurrence only
            new_content = re.replace(&content, &version_header).to_string();
            found = true;
            debug!("Matched pattern: {pattern} - replacing with: {version_header}");
            break;
        }
    }

    if !found {
        warn!(
            "No unreleased section found in changelog at {}",
            path.display()
        );
        // We'll just keep the file as is to avoid incorrect modifications
    } else {
        // Write the updated content back to the file
        fs::write(path, new_content).context("Failed to write updated changelog")?;
        debug!(
            "Updated unreleased section to version {} in {}",
            version,
            path.display()
        );
    }

    Ok(())
}

/// Preview the changelog update without making changes (dry run)
pub fn dry_run_update_changelog(path: &Path, version: &Version) -> Result<String> {
    let content = fs::read_to_string(path).context("Failed to read changelog file")?;

    let today = Local::now().format("%Y-%m-%d").to_string();
    let mut diff = String::new();
    let mut found = false;

    let version_header = format!("## [{version}] - {today}");

    // Try each pattern until one matches
    for pattern in &UNRELEASED_PATTERNS {
        // Compile the regex with proper flags
        let re = match Regex::new(pattern) {
            Ok(re) => re,
            Err(e) => {
                warn!("Invalid regex pattern: {pattern} - {e}");
                continue;
            }
        };

        if let Some(m) = re.find(&content) {
            let header = m.as_str();

            diff = format!(
                "Would update changelog {}:\n  {} → {}",
                path.display(),
                header.trim(),
                version_header
            );
            found = true;
            break;
        }
    }

    if !found {
        diff = format!(
            "No unreleased section found in changelog {}",
            path.display()
        );
    }

    Ok(diff)
}
