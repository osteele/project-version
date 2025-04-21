mod project;
mod changelog;
mod git;

use std::path::Path;
use clap::{Parser, Subcommand, ValueEnum};
use anyhow::{Context, Result};
use colored::Colorize;
use log::debug;

#[derive(Debug, Copy, Clone, ValueEnum)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

#[derive(Parser, Debug)]
#[command(
    name = "project-version",
    about = "Cross-language project version bumper for multiple project types",
    version,
    author
)]
struct Cli {
    /// Verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Dry run (no file modifications or git operations)
    #[arg(short = 'n', long, global = true)]
    dry_run: bool,

    /// Project directory to bump (defaults to current directory)
    #[arg(default_value = ".", global = true)]
    directory: String,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bump project version (major, minor, or patch)
    Bump {
        /// Type of version bump to perform
        #[arg(value_enum, default_value = "patch")]
        bump_type: BumpType,

        /// Skip committing changes
        #[arg(long)]
        no_commit: bool,

        /// Skip tagging the commit
        #[arg(long)]
        no_tag: bool,

        /// Force tag creation (overwrite existing tag)
        #[arg(long)]
        force_tag: bool,
    },
}

fn main() -> Result<()> {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse CLI arguments
    let args = Cli::parse();
    
    if args.verbose {
        println!("{}", "project-version - Cross-language version bumper".green().bold());
        debug!("Arguments: {:?}", args);
    }

    if args.dry_run {
        println!("{}", "[DRY RUN] No files will be modified".yellow());
    }

    // Find the project file
    let project = project::detect_project(&args.directory)
        .context("Failed to detect project type")?;
    
    // Get current version
    let current_version = project.get_version()?;
    if args.verbose {
        println!("Current version: {}", current_version);
    }

    match &args.command {
        Some(Commands::Bump { bump_type, no_commit, no_tag, force_tag }) => {
            // Handle the bump subcommand
            bump_version(
                project.as_ref(), 
                current_version, 
                *bump_type,
                args.dry_run,
                args.verbose,
                *no_commit,
                *no_tag,
                *force_tag,
                &args.directory
            )?;
        }
        None => {
            // If no subcommand is provided, just display current version
            println!("Current version: {}", current_version.to_string().blue());
            println!("\nUse 'project-version bump' to bump the version");
            println!("Run 'project-version --help' to see available commands");
        }
    }
    
    Ok(())
}

fn bump_version(
    project: &dyn project::Project,
    current_version: semver::Version,
    bump_type: BumpType,
    dry_run: bool,
    verbose: bool,
    no_commit: bool,
    no_tag: bool,
    force_tag: bool,
    directory: &str,
) -> Result<()> {
    // Calculate new version
    let new_version = match bump_type {
        BumpType::Major => {
            semver::Version::new(
                current_version.major + 1,
                0,
                0,
            )
        },
        BumpType::Minor => {
            semver::Version::new(
                current_version.major,
                current_version.minor + 1,
                0,
            )
        },
        BumpType::Patch => {
            semver::Version::new(
                current_version.major,
                current_version.minor,
                current_version.patch + 1,
            )
        },
    };
    
    println!("Bumping version: {} → {}", 
        current_version.to_string().blue(), 
        new_version.to_string().green().bold());

    // Update project file
    if dry_run {
        let diff = project.dry_run_update(&new_version)?;
        println!("{} {}", "[DRY RUN]".yellow(), diff);
    } else {
        project.update_version(&new_version)?;
        if verbose {
            println!("Updated {} with new version", project.get_file_path().display());
        }
    }
    
    // Check for CHANGELOG
    let changelog_path = changelog::find_changelog(directory);
    let has_changelog = changelog_path.is_some();
    
    if let Some(ref changelog_path) = changelog_path {
        if verbose {
            println!("Found changelog at {}", changelog_path.display());
        }
        
        if dry_run {
            let diff = changelog::dry_run_update_changelog(changelog_path, &new_version)?;
            println!("{} {}", "[DRY RUN]".yellow(), diff);
        } else {
            changelog::update_changelog(changelog_path, &new_version)?;
            println!("Updated changelog: {}", changelog_path.display());
        }
    }
    
    // Update lock files with the appropriate package manager
    if !dry_run {
        if let Some(update_command) = project.get_package_manager_update_command() {
            println!("Updating dependencies with: {}", update_command.cyan());
            
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&update_command)
                .current_dir(project.get_file_path().parent().unwrap_or(Path::new(".")))
                .output();
                
            match output {
                Ok(output) => {
                    if output.status.success() {
                        if verbose {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if !stdout.is_empty() {
                                println!("Package manager output:\n{}", stdout);
                            }
                        }
                        println!("Successfully updated dependencies");
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("Failed to update dependencies: {}", stderr);
                    }
                },
                Err(e) => {
                    eprintln!("Failed to run package manager: {}", e);
                }
            }
        }
    } else if let Some(update_command) = project.get_package_manager_update_command() {
        println!("{} Would update dependencies with: {}", "[DRY RUN]".yellow(), update_command);
    }
    
    // Git operations
    if !no_commit && !dry_run {
        let files_to_commit = project.get_files_to_commit();
        if has_changelog {
            let mut files = files_to_commit;
            // Safe to unwrap since we checked has_changelog
            files.push(changelog_path.unwrap());
            git::commit_changes(&files, &format!("release: version {}", new_version))?;
        } else {
            git::commit_changes(&files_to_commit, &format!("release: version {}", new_version))?;
        }
        println!("Committed version bump");
        
        if !no_tag {
            let tag_name = format!("v{}", new_version);
            
            // Check if tag exists
            if git::tag_exists(&tag_name)? {
                if force_tag {
                    git::create_tag(&tag_name, true)?;
                    println!("Forced creation of tag: {}", tag_name.green());
                } else {
                    // Prompt user to overwrite
                    use dialoguer::{theme::ColorfulTheme, Confirm};
                    
                    let overwrite = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Tag {} already exists. Overwrite?", tag_name))
                        .default(false)
                        .interact()?;
                        
                    if overwrite {
                        git::create_tag(&tag_name, true)?;
                        println!("Overwrote existing tag: {}", tag_name.green());
                    } else {
                        println!("Skipped tag creation (tag already exists)");
                    }
                }
            } else {
                git::create_tag(&tag_name, false)?;
                println!("Created tag: {}", tag_name.green());
            }
        }
    } else if dry_run {
        println!("{} Would commit changes and create tag v{}", "[DRY RUN]".yellow(), new_version);
    }
    
    Ok(())
}