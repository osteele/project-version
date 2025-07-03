mod changelog;
mod git;
mod project;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use colored::Colorize;
use log::debug;
use std::path::Path;

#[derive(Debug, Copy, Clone, ValueEnum)]
enum BumpType {
    Major,
    Minor,
    Patch,
}

/// Configuration for version operations
#[derive(Debug, Clone)]
struct VersionConfig {
    dry_run: bool,
    verbose: bool,
    no_commit: bool,
    no_tag: bool,
    force_tag: bool,
    directory: String,
}

/// Configuration specific to the set version operation
#[derive(Debug, Clone)]
struct SetVersionConfig {
    config: VersionConfig,
    force: bool,
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
    /// Set project version to a specific version number
    Set {
        /// Version number to set (must be a valid semver string)
        version: String,

        /// Skip committing changes
        #[arg(long)]
        no_commit: bool,

        /// Skip tagging the commit
        #[arg(long)]
        no_tag: bool,

        /// Force tag creation (overwrite existing tag)
        #[arg(long)]
        force_tag: bool,

        /// Force setting version even if it's lower than current version
        #[arg(long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Parse CLI arguments
    let args = Cli::parse();

    if args.verbose {
        println!(
            "{}",
            "project-version - Cross-language version bumper"
                .green()
                .bold()
        );
        debug!("Arguments: {args:?}");
    }

    if args.dry_run {
        println!("{}", "[DRY RUN] No files will be modified".yellow());
    }

    // Find the project file
    let project =
        project::detect_project(&args.directory).context("Failed to detect project type")?;

    // Get current version
    let current_version = project.get_version()?;
    if args.verbose {
        println!("Current version: {current_version}");
    }

    match &args.command {
        Some(Commands::Bump {
            bump_type,
            no_commit,
            no_tag,
            force_tag,
        }) => {
            // Handle the bump subcommand
            let config = VersionConfig {
                dry_run: args.dry_run,
                verbose: args.verbose,
                no_commit: *no_commit,
                no_tag: *no_tag,
                force_tag: *force_tag,
                directory: args.directory.clone(),
            };
            bump_version(project.as_ref(), current_version, *bump_type, config)?
        }
        Some(Commands::Set {
            version,
            no_commit,
            no_tag,
            force_tag,
            force,
        }) => {
            // Handle the set subcommand
            let config = SetVersionConfig {
                config: VersionConfig {
                    dry_run: args.dry_run,
                    verbose: args.verbose,
                    no_commit: *no_commit,
                    no_tag: *no_tag,
                    force_tag: *force_tag,
                    directory: args.directory.clone(),
                },
                force: *force,
            };
            set_version(project.as_ref(), current_version, version, config)?
        }
        None => {
            // If no subcommand is provided, just display current version
            println!("Current version: {}", current_version.to_string().blue());
            println!("\nUse 'project-version bump' to bump the version");
            println!("Use 'project-version set <VERSION>' to set a specific version");
            println!("Run 'project-version --help' to see available commands");
        }
    }

    Ok(())
}

fn bump_version(
    project: &dyn project::Project,
    current_version: semver::Version,
    bump_type: BumpType,
    config: VersionConfig,
) -> Result<()> {
    // Calculate new version
    let new_version = match bump_type {
        BumpType::Major => semver::Version::new(current_version.major + 1, 0, 0),
        BumpType::Minor => {
            semver::Version::new(current_version.major, current_version.minor + 1, 0)
        }
        BumpType::Patch => semver::Version::new(
            current_version.major,
            current_version.minor,
            current_version.patch + 1,
        ),
    };

    println!(
        "Bumping version: {} → {}",
        current_version.to_string().blue(),
        new_version.to_string().green().bold()
    );

    // Update project file
    if config.dry_run {
        let diff = project.dry_run_update(&new_version)?;
        println!("{} {}", "[DRY RUN]".yellow(), diff);
    } else {
        project.update_version(&new_version)?;
        if config.verbose {
            println!(
                "Updated {} with new version",
                project.get_file_path().display()
            );
        }
    }

    // Check for CHANGELOG
    let changelog_path = changelog::find_changelog(&config.directory);
    let has_changelog = changelog_path.is_some();

    if let Some(ref changelog_path) = changelog_path {
        if config.verbose {
            println!("Found changelog at {}", changelog_path.display());
        }

        if config.dry_run {
            let diff = changelog::dry_run_update_changelog(changelog_path, &new_version)?;
            println!("{} {}", "[DRY RUN]".yellow(), diff);
        } else {
            changelog::update_changelog(changelog_path, &new_version)?;
            println!("Updated changelog: {}", changelog_path.display());
        }
    }

    // Update lock files with the appropriate package manager
    if !config.dry_run {
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
                        if config.verbose {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if !stdout.is_empty() {
                                println!("Package manager output:\n{stdout}");
                            }
                        }
                        println!("Successfully updated dependencies");
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("Failed to update dependencies: {stderr}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to run package manager: {e}");
                }
            }
        }
    } else if let Some(update_command) = project.get_package_manager_update_command() {
        println!(
            "{} Would update dependencies with: {}",
            "[DRY RUN]".yellow(),
            update_command
        );
    }

    // Git operations
    if !config.no_commit && !config.dry_run {
        let files_to_commit = project.get_files_to_commit();
        if has_changelog {
            let mut files = files_to_commit;
            // Safe to unwrap since we checked has_changelog
            files.push(changelog_path.unwrap());
            git::commit_changes(&files, &format!("release: version {new_version}"))?;
        } else {
            git::commit_changes(&files_to_commit, &format!("release: version {new_version}"))?;
        }
        println!("Committed version bump");

        if !config.no_tag {
            let tag_name = format!("v{new_version}");

            // Check if tag exists
            if git::tag_exists(&tag_name)? {
                if config.force_tag {
                    git::create_tag(&tag_name, true)?;
                    println!("Forced creation of tag: {}", tag_name.green());
                } else {
                    // Prompt user to overwrite
                    use dialoguer::{theme::ColorfulTheme, Confirm};

                    let overwrite = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Tag {tag_name} already exists. Overwrite?"))
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
    } else if config.dry_run {
        println!(
            "{} Would commit changes and create tag v{}",
            "[DRY RUN]".yellow(),
            new_version
        );
    }

    Ok(())
}

fn set_version(
    project: &dyn project::Project,
    current_version: semver::Version,
    version_str: &str,
    config: SetVersionConfig,
) -> Result<()> {
    // Parse the provided version string, stripping a leading 'v' if present
    let clean_version_str = version_str.strip_prefix('v').unwrap_or(version_str);

    let new_version = semver::Version::parse(clean_version_str)
        .context(format!("Invalid version format: {version_str}"))?;

    // Check if the new version is lower than the current version
    if !config.force && new_version < current_version {
        return Err(anyhow!("New version ({}) is lower than current version ({}). Use --force to override this check.", 
            new_version, current_version));
    }

    println!(
        "Setting version: {} → {}",
        current_version.to_string().blue(),
        new_version.to_string().green().bold()
    );

    // Update project file
    if config.config.dry_run {
        let diff = project.dry_run_update(&new_version)?;
        println!("{} {}", "[DRY RUN]".yellow(), diff);
    } else {
        project.update_version(&new_version)?;
        if config.config.verbose {
            println!(
                "Updated {} with new version",
                project.get_file_path().display()
            );
        }
    }

    // Check for CHANGELOG
    let changelog_path = changelog::find_changelog(&config.config.directory);
    let has_changelog = changelog_path.is_some();

    if let Some(ref changelog_path) = changelog_path {
        if config.config.verbose {
            println!("Found changelog at {}", changelog_path.display());
        }

        if config.config.dry_run {
            let diff = changelog::dry_run_update_changelog(changelog_path, &new_version)?;
            println!("{} {}", "[DRY RUN]".yellow(), diff);
        } else {
            changelog::update_changelog(changelog_path, &new_version)?;
            println!("Updated changelog: {}", changelog_path.display());
        }
    }

    // Update lock files with the appropriate package manager
    if !config.config.dry_run {
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
                        if config.config.verbose {
                            let stdout = String::from_utf8_lossy(&output.stdout);
                            if !stdout.is_empty() {
                                println!("Package manager output:\n{stdout}");
                            }
                        }
                        println!("Successfully updated dependencies");
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr);
                        eprintln!("Failed to update dependencies: {stderr}");
                    }
                }
                Err(e) => {
                    eprintln!("Failed to run package manager: {e}");
                }
            }
        }
    } else if let Some(update_command) = project.get_package_manager_update_command() {
        println!(
            "{} Would update dependencies with: {}",
            "[DRY RUN]".yellow(),
            update_command
        );
    }

    // Git operations
    if !config.config.no_commit && !config.config.dry_run {
        let files_to_commit = project.get_files_to_commit();
        if has_changelog {
            let mut files = files_to_commit;
            // Safe to unwrap since we checked has_changelog
            files.push(changelog_path.unwrap());
            git::commit_changes(&files, &format!("release: version {new_version}"))?;
        } else {
            git::commit_changes(&files_to_commit, &format!("release: version {new_version}"))?;
        }
        println!("Committed version bump");

        if !config.config.no_tag {
            let tag_name = format!("v{new_version}");

            // Check if tag exists
            if git::tag_exists(&tag_name)? {
                if config.config.force_tag {
                    git::create_tag(&tag_name, true)?;
                    println!("Forced creation of tag: {}", tag_name.green());
                } else {
                    // Prompt user to overwrite
                    use dialoguer::{theme::ColorfulTheme, Confirm};

                    let overwrite = Confirm::with_theme(&ColorfulTheme::default())
                        .with_prompt(format!("Tag {tag_name} already exists. Overwrite?",))
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
    } else if config.config.dry_run {
        println!(
            "{} Would commit changes and create tag v{}",
            "[DRY RUN]".yellow(),
            new_version
        );
    }

    Ok(())
}
