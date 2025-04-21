mod project;
mod changelog;
mod git;

use clap::{Parser, ValueEnum};
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
    /// Type of version bump to perform
    #[arg(value_enum, default_value = "patch")]
    bump: BumpType,

    /// Skip committing changes
    #[arg(long)]
    no_commit: bool,

    /// Skip tagging the commit
    #[arg(long)]
    no_tag: bool,

    /// Force tag creation (overwrite existing tag)
    #[arg(long)]
    force_tag: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Dry run (no file modifications or git operations)
    #[arg(short = 'n', long)]
    dry_run: bool,

    /// Project directory to bump (defaults to current directory)
    #[arg(default_value = ".")]
    directory: String,
}

fn main() -> Result<()> {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse CLI arguments
    let args = Cli::parse();
    
    if args.verbose {
        println!("{}", "polybump - Cross-language version bumper".green().bold());
        debug!("Arguments: {:?}", args);
    }

    if args.dry_run {
        println!("{}", "[DRY RUN] No files will be modified".yellow());
    }

    // Find the project file
    let project = project::detect_project(&args.directory)
        .context("Failed to detect project type")?;
    
    let current_version = project.get_version()?;
    if args.verbose {
        println!("Current version: {}", current_version);
    }

    // Calculate new version
    let new_version = match args.bump {
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
    if args.dry_run {
        let diff = project.dry_run_update(&new_version)?;
        println!("{} {}", "[DRY RUN]".yellow(), diff);
    } else {
        project.update_version(&new_version)?;
        if args.verbose {
            println!("Updated {} with new version", project.get_file_path().display());
        }
    }
    
    // Check for CHANGELOG
    let changelog_path = changelog::find_changelog(&args.directory);
    let has_changelog = changelog_path.is_some();
    
    if let Some(ref changelog_path) = changelog_path {
        if args.verbose {
            println!("Found changelog at {}", changelog_path.display());
        }
        
        if args.dry_run {
            let diff = changelog::dry_run_update_changelog(changelog_path, &new_version)?;
            println!("{} {}", "[DRY RUN]".yellow(), diff);
        } else {
            changelog::update_changelog(changelog_path, &new_version)?;
            println!("Updated changelog: {}", changelog_path.display());
        }
    }
    
    // Git operations
    if !args.no_commit && !args.dry_run {
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
        
        if !args.no_tag {
            let tag_name = format!("v{}", new_version);
            
            // Check if tag exists
            if git::tag_exists(&tag_name)? {
                if args.force_tag {
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
    } else if args.dry_run {
        println!("{} Would commit changes and create tag v{}", "[DRY RUN]".yellow(), new_version);
    }
    
    Ok(())
}