use anyhow::{anyhow, Context, Result};
use log::{debug, warn};
use std::path::PathBuf;
use std::process::Command;

// Check if the current directory is a git repository
pub fn is_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

// Commit changes to the specified files
pub fn commit_changes(files: &[PathBuf], message: &str) -> Result<()> {
    if !is_git_repo() {
        warn!("Not a git repository, skipping commit");
        return Ok(());
    }

    // Add files to staging area
    for file in files {
        debug!("Staging file: {}", file.display());

        let status = Command::new("git")
            .args(["add", &file.to_string_lossy()])
            .status()
            .context("Failed to run git add command")?;

        if !status.success() {
            return Err(anyhow!(
                "Failed to add file to git staging area: {}",
                file.display()
            ));
        }
    }

    // Commit changes
    debug!("Committing with message: {}", message);

    let status = Command::new("git")
        .args(["commit", "-m", message])
        .status()
        .context("Failed to run git commit command")?;

    if !status.success() {
        return Err(anyhow!("Git commit failed"));
    }

    Ok(())
}

// Check if a tag exists
pub fn tag_exists(tag: &str) -> Result<bool> {
    if !is_git_repo() {
        warn!("Not a git repository, assuming tag doesn't exist");
        return Ok(false);
    }

    let output = Command::new("git")
        .args(["tag", "-l", tag])
        .output()
        .context("Failed to run git tag command")?;

    Ok(!output.stdout.is_empty())
}

// Create a tag
pub fn create_tag(tag: &str, force: bool) -> Result<()> {
    if !is_git_repo() {
        warn!("Not a git repository, skipping tag creation");
        return Ok(());
    }

    let mut args = vec!["tag"];

    if force {
        args.push("-f");
    }

    args.push(tag);

    debug!("Creating git tag: {}", tag);

    let status = Command::new("git")
        .args(&args)
        .status()
        .context("Failed to run git tag command")?;

    if !status.success() {
        return Err(anyhow!("Failed to create git tag: {}", tag));
    }

    Ok(())
}
