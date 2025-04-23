use anyhow::{anyhow, Context, Result};
use log::{debug, warn};
use semver::Version;
use std::fs;
use std::path::{Path, PathBuf};

pub trait Project {
    fn get_version(&self) -> Result<Version>;

    /// Update the version in the project file
    fn update_version(&self, version: &Version) -> Result<()>;

    /// Preview what would be updated without making changes
    fn dry_run_update(&self, version: &Version) -> Result<String>;

    /// Get the path to the main project file
    fn get_file_path(&self) -> &Path;

    /// Get all files that should be committed
    fn get_files_to_commit(&self) -> Vec<PathBuf>;

    /// Get the package manager update command for this project
    fn get_package_manager_update_command(&self) -> Option<String> {
        None
    }
}

pub fn detect_project(dir: &str) -> Result<Box<dyn Project>> {
    let dir_path = Path::new(dir);

    // Check for package.json (Node.js)
    let package_json_path = dir_path.join("package.json");
    if package_json_path.exists() {
        debug!("Detected Node.js project (package.json)");
        return Ok(Box::new(NodeProject::new(package_json_path)));
    }

    // Check for pyproject.toml (Python)
    let pyproject_path = dir_path.join("pyproject.toml");
    if pyproject_path.exists() {
        debug!("Detected Python project (pyproject.toml)");
        return Ok(Box::new(PythonProject::new(pyproject_path)));
    }

    // Check for Cargo.toml (Rust)
    let cargo_path = dir_path.join("Cargo.toml");
    if cargo_path.exists() {
        debug!("Detected Rust project (Cargo.toml)");
        return Ok(Box::new(RustProject::new(cargo_path)));
    }

    // Check for Go module (go.mod)
    let go_mod_path = dir_path.join("go.mod");
    if go_mod_path.exists() {
        debug!("Detected Go project (go.mod)");
        return Ok(Box::new(GoProject::new(go_mod_path)));
    }

    // Check for Gemfile (Ruby)
    let gemfile_path = dir_path.join("Gemfile");
    if gemfile_path.exists() {
        debug!("Detected Ruby project (Gemfile)");
        return Ok(Box::new(RubyProject::new(gemfile_path)));
    }

    Err(anyhow!("No supported project files found in {}", dir))
}

// Node.js project (package.json)
pub struct NodeProject {
    path: PathBuf,
}

impl NodeProject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // Internal function that does the actual work, can be dry-run or real update
    fn update_version_internal(&self, version: &Version, dry_run: bool) -> Result<String> {
        // Read the original content
        let content = fs::read_to_string(&self.path).context("Failed to read package.json")?;

        let old_version = self.get_version()?;

        // Using regex for targeted replacement that preserves all formatting
        // We considered JSON parsing libraries but:
        // - serde_json (even with preserve_order) loses comments and exact formatting
        // - json-patch (RFC 6902) doesn't preserve whitespace or formatting
        // - No suitable Rust crate exists that preserves comments and exact formatting
        let re = regex::Regex::new(r#"("version"\s*:\s*")([^"]*)(")"#).unwrap();
        let new_content = re.replace(&content, |caps: &regex::Captures| {
            format!("{}{}{}", &caps[1], version, &caps[3])
        });

        let diff = format!(
            "{}:\n  version: {} → {}",
            if dry_run {
                "Would update package.json"
            } else {
                "Updated package.json"
            },
            old_version,
            version
        );

        if !dry_run {
            fs::write(&self.path, new_content.as_bytes())
                .context("Failed to write updated package.json")?;
        }

        Ok(diff)
    }
}

impl Project for NodeProject {
    fn get_version(&self) -> Result<Version> {
        let content = fs::read_to_string(&self.path).context("Failed to read package.json")?;

        let package: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse package.json")?;

        let version_str = package["version"]
            .as_str()
            .ok_or_else(|| anyhow!("No version field found in package.json"))?;

        Version::parse(version_str).context("Failed to parse version from package.json")
    }

    fn update_version(&self, version: &Version) -> Result<()> {
        self.update_version_internal(version, false)?;
        Ok(())
    }

    fn dry_run_update(&self, version: &Version) -> Result<String> {
        self.update_version_internal(version, true)
    }

    fn get_file_path(&self) -> &Path {
        &self.path
    }

    fn get_files_to_commit(&self) -> Vec<PathBuf> {
        vec![self.path.clone()]
    }

    fn get_package_manager_update_command(&self) -> Option<String> {
        let dir = self.path.parent().unwrap_or(Path::new("."));

        // Check for different lock files to detect the package manager
        if dir.join("bun.lockb").exists() {
            return Some("bun install".to_string());
        } else if dir.join("yarn.lock").exists() {
            return Some("yarn".to_string());
        } else if dir.join("pnpm-lock.yaml").exists() {
            return Some("pnpm install".to_string());
        } else if dir.join("package-lock.json").exists() {
            return Some("npm install".to_string());
        }

        // Default to npm if no lock file is found
        Some("npm install".to_string())
    }
}

// Python project (pyproject.toml)
pub struct PythonProject {
    path: PathBuf,
}

impl PythonProject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // Helper to find the version location in pyproject.toml
    fn find_version_locations(&self, content: &str) -> Result<Vec<(String, String)>> {
        let toml_value: toml::Value = content.parse().context("Failed to parse pyproject.toml")?;

        let mut locations = Vec::new();

        // Check project.version (PEP 621)
        if let Some(project) = toml_value.get("project").and_then(|p| p.as_table()) {
            if let Some(version) = project.get("version").and_then(|v| v.as_str()) {
                locations.push(("project.version".to_string(), version.to_string()));
            }
        }

        // Check tool.poetry.version (Poetry)
        if let Some(tool) = toml_value.get("tool").and_then(|t| t.as_table()) {
            if let Some(poetry) = tool.get("poetry").and_then(|p| p.as_table()) {
                if let Some(version) = poetry.get("version").and_then(|v| v.as_str()) {
                    locations.push(("tool.poetry.version".to_string(), version.to_string()));
                }
            }
        }

        // Check other common locations
        if let Some(tool) = toml_value.get("tool").and_then(|t| t.as_table()) {
            if let Some(setuptools) = tool.get("setuptools").and_then(|s| s.as_table()) {
                if let Some(version) = setuptools.get("version").and_then(|v| v.as_str()) {
                    locations.push(("tool.setuptools.version".to_string(), version.to_string()));
                }
            }
        }

        if locations.is_empty() {
            return Err(anyhow!("No version field found in pyproject.toml"));
        }

        Ok(locations)
    }
}

impl PythonProject {
    fn update_version_internal(&self, version: &Version, dry_run: bool) -> Result<String> {
        // Read the file content
        let content = fs::read_to_string(&self.path).context("Failed to read pyproject.toml")?;

        // Find all version locations for reporting
        let locations = self.find_version_locations(&content)?;

        let prefix = if dry_run { "Would update" } else { "Updated" };
        let mut diff = format!("{} pyproject.toml:", prefix);

        for (location, old_version) in &locations {
            diff.push_str(&format!("\n  {}: {} → {}", location, old_version, version));
        }

        if !dry_run {
            // Parse the TOML with toml_edit to preserve formatting, spacing, and comments
            let mut doc = match content.parse::<toml_edit::DocumentMut>() {
                Ok(doc) => doc,
                Err(e) => return Err(anyhow!("Failed to parse pyproject.toml: {}", e)),
            };

            // Update each known version location
            let version_str = version.to_string();

            // Check project.version (PEP 621)
            if let Some(project) = doc.get_mut("project") {
                if let Some(project_table) = project.as_table_mut() {
                    if project_table.contains_key("version") {
                        project_table["version"] = toml_edit::value(version_str.clone());
                    }
                }
            }

            // Check tool.poetry.version
            if let Some(tool) = doc.get_mut("tool") {
                if let Some(tool_table) = tool.as_table_mut() {
                    if let Some(poetry) = tool_table.get_mut("poetry") {
                        if let Some(poetry_table) = poetry.as_table_mut() {
                            if poetry_table.contains_key("version") {
                                poetry_table["version"] = toml_edit::value(version_str.clone());
                            }
                        }
                    }

                    // Check tool.setuptools.version
                    if let Some(setuptools) = tool_table.get_mut("setuptools") {
                        if let Some(setuptools_table) = setuptools.as_table_mut() {
                            if setuptools_table.contains_key("version") {
                                setuptools_table["version"] = toml_edit::value(version_str.clone());
                            }
                        }
                    }
                }
            }

            // Write the updated TOML
            let new_content = doc.to_string();
            if new_content == content {
                warn!("No version patterns matched in pyproject.toml");
                return Err(anyhow!("Failed to update version in pyproject.toml"));
            }

            fs::write(&self.path, new_content).context("Failed to write updated pyproject.toml")?;
        }

        Ok(diff)
    }
}

impl Project for PythonProject {
    fn get_version(&self) -> Result<Version> {
        let content = fs::read_to_string(&self.path).context("Failed to read pyproject.toml")?;

        // Find all version locations
        let locations = self.find_version_locations(&content)?;

        // Use the first one
        let version_str = &locations[0].1;

        Version::parse(version_str).context("Failed to parse version from pyproject.toml")
    }

    fn update_version(&self, version: &Version) -> Result<()> {
        self.update_version_internal(version, false)?;
        Ok(())
    }

    fn dry_run_update(&self, version: &Version) -> Result<String> {
        self.update_version_internal(version, true)
    }

    fn get_file_path(&self) -> &Path {
        &self.path
    }

    fn get_files_to_commit(&self) -> Vec<PathBuf> {
        vec![self.path.clone()]
    }

    fn get_package_manager_update_command(&self) -> Option<String> {
        let dir = self.path.parent().unwrap_or(Path::new("."));

        // Check for different lock files and configurations to detect the package manager
        if dir.join("poetry.lock").exists() {
            return Some("poetry update".to_string());
        } else if dir.join("Pipfile.lock").exists() {
            return Some("pipenv update".to_string());
        } else if dir.join("pdm.lock").exists() {
            return Some("pdm update".to_string());
        }

        // Check for the presence of uv files or directories
        if dir.join("uv.lock").exists() || dir.join(".uv").exists() {
            return Some("uv sync".to_string());
        }

        // Read the content of the pyproject.toml to detect Python package tool
        if let Ok(content) = fs::read_to_string(&self.path) {
            if content.contains("[tool.poetry]") {
                return Some("poetry update".to_string());
            } else if content.contains("[tool.pdm]") {
                return Some("pdm update".to_string());
            } else if content.contains("[tool.hatch") {
                return Some("hatch env update".to_string());
            }
        }

        // Default to pip
        if dir.join("requirements.txt").exists() {
            return Some("pip install -r requirements.txt".to_string());
        }

        // Return None if we can't confidently determine the package manager
        None
    }
}

// Rust project (Cargo.toml)
pub struct RustProject {
    path: PathBuf,
}

impl RustProject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl RustProject {
    fn update_version_internal(&self, version: &Version, dry_run: bool) -> Result<String> {
        let content = fs::read_to_string(&self.path).context("Failed to read Cargo.toml")?;

        let old_version = self.get_version()?;

        let prefix = if dry_run { "Would update" } else { "Updated" };
        let diff = format!(
            "{} Cargo.toml:\n  version: {} → {}",
            prefix, old_version, version
        );

        if !dry_run {
            // Parse the TOML with toml_edit to preserve formatting, spacing, and comments
            let mut doc = match content.parse::<toml_edit::DocumentMut>() {
                Ok(doc) => doc,
                Err(e) => return Err(anyhow!("Failed to parse Cargo.toml: {}", e)),
            };

            // Update the package.version
            if let Some(package) = doc.get_mut("package") {
                if let Some(package_table) = package.as_table_mut() {
                    if package_table.contains_key("version") {
                        package_table["version"] = toml_edit::value(version.to_string());
                    } else {
                        return Err(anyhow!(
                            "No version field found in Cargo.toml package table"
                        ));
                    }
                }
            } else {
                return Err(anyhow!("No package table found in Cargo.toml"));
            }

            // Write the updated TOML
            let new_content = doc.to_string();
            fs::write(&self.path, new_content).context("Failed to write updated Cargo.toml")?;
        }

        Ok(diff)
    }
}

impl Project for RustProject {
    fn get_version(&self) -> Result<Version> {
        let content = fs::read_to_string(&self.path).context("Failed to read Cargo.toml")?;

        let toml_value: toml::Value = content.parse().context("Failed to parse Cargo.toml")?;

        let version_str = toml_value
            .get("package")
            .and_then(|package| package.get("version"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No version field found in Cargo.toml"))?;

        Version::parse(version_str).context("Failed to parse version from Cargo.toml")
    }

    fn update_version(&self, version: &Version) -> Result<()> {
        self.update_version_internal(version, false)?;
        Ok(())
    }

    fn dry_run_update(&self, version: &Version) -> Result<String> {
        self.update_version_internal(version, true)
    }

    fn get_file_path(&self) -> &Path {
        &self.path
    }

    fn get_files_to_commit(&self) -> Vec<PathBuf> {
        vec![self.path.clone()]
    }

    fn get_package_manager_update_command(&self) -> Option<String> {
        // Cargo is the only package manager for Rust
        Some("cargo update".to_string())
    }
}

// Go project (go.mod)
pub struct GoProject {
    path: PathBuf,
}

impl GoProject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl GoProject {
    fn get_version_files(&self) -> Vec<PathBuf> {
        let dir = self.path.parent().unwrap_or(Path::new("."));
        let candidates = [
            dir.join("version.go"),
            dir.join("internal/version/version.go"),
            dir.join("pkg/version/version.go"),
        ];

        candidates
            .into_iter()
            .filter(|path| path.exists())
            .collect()
    }

    fn update_version_internal(&self, version: &Version, dry_run: bool) -> Result<String> {
        let version_files = self.get_version_files();
        let mut updated_files = Vec::new();
        let mut diff = String::new();

        let old_version = match self.get_version() {
            Ok(v) => v,
            Err(_) => Version::new(0, 1, 0),
        };

        diff.push_str(&format!(
            "{} Go project version from {} to {}:\n",
            if dry_run { "Would update" } else { "Updated" },
            old_version,
            version
        ));

        if version_files.is_empty() {
            diff.push_str("  No version files found. You may need to manually create/update version information.");
            return Ok(diff);
        }

        let version_regex = regex::Regex::new(
            r#"((?:Version|VERSION)\s*=\s*["'])v?([0-9]+\.[0-9]+\.[0-9]+)(["'])"#,
        )
        .unwrap();

        for file_path in &version_files {
            let file_content =
                fs::read_to_string(file_path).context("Failed to read version file")?;

            if version_regex.is_match(&file_content) {
                diff.push_str(&format!("  File: {}\n", file_path.display()));

                let new_content = version_regex.replace(&file_content, |caps: &regex::Captures| {
                    format!("{}v{}{}", &caps[1], version, &caps[3])
                });

                if !dry_run && new_content != file_content {
                    fs::write(file_path, new_content.as_bytes())
                        .context("Failed to write updated version file")?;
                    updated_files.push(file_path);
                }
            }
        }

        if !dry_run && updated_files.is_empty() {
            warn!("No version file was updated for Go project.");
            diff.push_str("  No version patterns were matched in any files.");
        }

        Ok(diff)
    }
}

impl Project for GoProject {
    fn get_version(&self) -> Result<Version> {
        let version_files = self.get_version_files();

        let version_regex =
            regex::Regex::new(r#"(?:Version|VERSION)\s*=\s*["']v?([0-9]+\.[0-9]+\.[0-9]+)["']"#)
                .unwrap();

        for file_path in &version_files {
            let version_content =
                fs::read_to_string(file_path).context("Failed to read version file")?;

            if let Some(captures) = version_regex.captures(&version_content) {
                if let Some(version_match) = captures.get(1) {
                    return Version::parse(version_match.as_str())
                        .context("Failed to parse version from version.go file");
                }
            }
        }

        // Fallback: use git tags
        warn!("Could not find version information in Go files, you may need to manually tag this version");

        // Default to 0.1.0 if we can't determine it
        Ok(Version::new(0, 1, 0))
    }

    fn update_version(&self, version: &Version) -> Result<()> {
        self.update_version_internal(version, false)?;
        Ok(())
    }

    fn dry_run_update(&self, version: &Version) -> Result<String> {
        self.update_version_internal(version, true)
    }

    fn get_file_path(&self) -> &Path {
        &self.path
    }

    fn get_files_to_commit(&self) -> Vec<PathBuf> {
        let mut files = vec![self.path.clone()];
        files.extend(self.get_version_files());
        files
    }

    fn get_package_manager_update_command(&self) -> Option<String> {
        // Go modules has a specific update command
        Some("go mod tidy".to_string())
    }
}

// Ruby project (Gemfile)
pub struct RubyProject {
    path: PathBuf,
}

impl RubyProject {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    // Get the version from gemspec file
    fn find_gemspec_file(&self) -> Option<PathBuf> {
        let dir = self.path.parent().unwrap_or(Path::new("."));

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.extension().is_some_and(|ext| ext == "gemspec") {
                    return Some(path);
                }
            }
        }
        None
    }

    // Try to extract version from version.rb file
    fn find_version_rb_file(&self) -> Option<PathBuf> {
        let dir = self.path.parent().unwrap_or(Path::new("."));

        // First approach: try to find the project name from gemspec
        if let Some(gemspec_path) = self.find_gemspec_file() {
            if let Ok(content) = fs::read_to_string(gemspec_path) {
                // Try to extract gem name from gemspec
                let name_re = regex::Regex::new(r#"[s\.]name\s*=\s*["']([^"']+)["']"#).unwrap();
                if let Some(caps) = name_re.captures(&content) {
                    if let Some(name) = caps.get(1) {
                        let version_path = dir.join(name.as_str()).join("version.rb");
                        if version_path.exists() {
                            return Some(version_path);
                        }
                    }
                }
            }
        }

        // Second approach: look for any version.rb file
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    let version_path = path.join("version.rb");
                    if version_path.exists() {
                        return Some(version_path);
                    }
                }
            }
        }

        // Third approach: direct version.rb in lib
        let direct_version_path = dir.join("lib/version.rb");
        if direct_version_path.exists() {
            return Some(direct_version_path);
        }

        None
    }

    fn update_version_internal(&self, version: &Version, dry_run: bool) -> Result<String> {
        // First check for gemspec file which usually contains the version
        let gemspec_path = self.find_gemspec_file();
        let version_rb_path = self.find_version_rb_file();

        let old_version = self.get_version()?;
        let mut diff = String::new();
        let mut updated_any = false;

        let prefix = if dry_run { "Would update" } else { "Updated" };
        diff.push_str(&format!(
            "{} Ruby project version from {} to {}:\n",
            prefix, old_version, version
        ));

        // Try to update gemspec
        if let Some(ref path) = gemspec_path {
            let content = fs::read_to_string(path).context("Failed to read gemspec file")?;

            let version_re = regex::Regex::new(r#"(['\"])(\d+\.\d+\.\d+)(['\"]\s*)"#).unwrap();

            if version_re.is_match(&content) {
                diff.push_str(&format!("  File: {}\n", path.display()));

                let new_content = version_re.replace(&content, |caps: &regex::Captures| {
                    format!("{}{}{}", &caps[1], version, &caps[3])
                });

                if !dry_run && new_content != content {
                    fs::write(path, new_content.as_bytes())
                        .context("Failed to write updated gemspec file")?;
                    updated_any = true;
                }
            }
        }

        // Try to update version.rb
        if let Some(ref path) = version_rb_path {
            let content = fs::read_to_string(path).context("Failed to read version.rb file")?;

            let version_re = regex::Regex::new(r#"VERSION\s*=\s*['"]([\d\.]+)['"](.*)"#).unwrap();

            if version_re.is_match(&content) {
                diff.push_str(&format!("  File: {}\n", path.display()));

                let new_content = version_re.replace(&content, |caps: &regex::Captures| {
                    format!(
                        "VERSION = '{}'{}",
                        version,
                        if let Some(m) = caps.get(2) {
                            m.as_str()
                        } else {
                            ""
                        }
                    )
                });

                if !dry_run && new_content != content {
                    fs::write(path, new_content.as_bytes())
                        .context("Failed to write updated version.rb file")?;
                    updated_any = true;
                }
            }
        }

        if !updated_any && !dry_run {
            warn!("Could not update Ruby project version. No version patterns were matched.");
            diff.push_str("  No version patterns were matched in any files.");
        }

        Ok(diff)
    }
}

impl Project for RubyProject {
    fn get_version(&self) -> Result<Version> {
        // Try to find version in gemspec
        if let Some(gemspec_path) = self.find_gemspec_file() {
            let content =
                fs::read_to_string(gemspec_path).context("Failed to read gemspec file")?;

            let version_re =
                regex::Regex::new(r#"(?:version|s\.version)\s*=\s*['"]([^'"]+)['"]\s*"#).unwrap();

            if let Some(caps) = version_re.captures(&content) {
                if let Some(version_match) = caps.get(1) {
                    return Version::parse(version_match.as_str())
                        .context("Failed to parse version from gemspec");
                }
            }
        }

        // Try to find version in version.rb
        if let Some(version_rb_path) = self.find_version_rb_file() {
            let content =
                fs::read_to_string(version_rb_path).context("Failed to read version.rb file")?;

            let version_re = regex::Regex::new(r#"VERSION\s*=\s*['"]([^'"]+)['"]\s*"#).unwrap();

            if let Some(caps) = version_re.captures(&content) {
                if let Some(version_match) = caps.get(1) {
                    return Version::parse(version_match.as_str())
                        .context("Failed to parse version from version.rb");
                }
            }
        }

        warn!("Could not find version information in Ruby project files");
        // Default to 0.1.0 if we can't determine it
        Ok(Version::new(0, 1, 0))
    }

    fn update_version(&self, version: &Version) -> Result<()> {
        self.update_version_internal(version, false)?;
        Ok(())
    }

    fn dry_run_update(&self, version: &Version) -> Result<String> {
        self.update_version_internal(version, true)
    }

    fn get_file_path(&self) -> &Path {
        &self.path
    }

    fn get_files_to_commit(&self) -> Vec<PathBuf> {
        let mut files = vec![self.path.clone()];

        if let Some(gemspec_path) = self.find_gemspec_file() {
            files.push(gemspec_path);
        }

        if let Some(version_rb_path) = self.find_version_rb_file() {
            files.push(version_rb_path);
        }

        files
    }

    fn get_package_manager_update_command(&self) -> Option<String> {
        let dir = self.path.parent().unwrap_or(Path::new("."));

        // Check for Gemfile.lock to confirm Bundler is used
        if dir.join("Gemfile.lock").exists() {
            return Some("bundle install".to_string());
        }

        // Default to bundle install if Gemfile exists
        Some("bundle install".to_string())
    }
}
