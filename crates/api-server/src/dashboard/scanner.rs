use anyhow::{Context, Result};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

use super::models::{Project, ProjectStatus};

pub struct ProjectScanner {
    base_path: PathBuf,
}

impl ProjectScanner {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Scan the base directory for projects
    pub async fn scan(&self) -> Result<Vec<Project>> {
        info!("📂 Scanning projects in {:?}", self.base_path);

        let mut projects = Vec::new();
        let mut entries = fs::read_dir(&self.base_path).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();

            // Skip hidden directories
            if let Some(name) = path.file_name() {
                if name.to_string_lossy().starts_with('.') {
                    continue;
                }
            }

            if path.is_dir() {
                match self.analyze_directory(&path).await {
                    Ok(Some(project)) => {
                        debug!("✓ Found project: {}", project.name);
                        projects.push(project);
                    }
                    Ok(None) => {
                        debug!("⊘ Skipped: {:?} (not a project)", path);
                    }
                    Err(e) => {
                        warn!("⚠ Error analyzing {:?}: {}", path, e);
                    }
                }
            }
        }

        info!("✓ Found {} projects", projects.len());
        Ok(projects)
    }

    /// Analyze a directory to determine if it's a project
    async fn analyze_directory(&self, path: &Path) -> Result<Option<Project>> {
        let name = path
            .file_name()
            .context("No filename")?
            .to_string_lossy()
            .to_string();

        // Check if it's a project directory (has git, cargo, package.json, etc.)
        let is_project = self.is_project_directory(path).await?;
        if !is_project {
            return Ok(None);
        }

        // Detect languages
        let languages = self.detect_languages(path).await?;

        // Get description from README if available
        let description = self.extract_description(path).await?;

        // Determine project status
        let status = self.determine_status(path).await?;

        // Get last commit time
        let last_commit = self.get_last_commit(path).await?;

        let project = Project {
            name: name.clone(),
            path: path.to_string_lossy().to_string(),
            description,
            languages,
            status,
            health_score: 0.0, // Will be calculated by analyzer
            last_commit,
            last_indexed: Some(chrono::Utc::now().to_rfc3339()),
            metadata: HashMap::new(),
        };

        Ok(Some(project))
    }

    /// Check if directory is a project
    async fn is_project_directory(&self, path: &Path) -> Result<bool> {
        // Check for common project indicators
        let indicators = [
            ".git",
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "go.mod",
            "pom.xml",
            "build.gradle",
        ];

        for indicator in indicators {
            if path.join(indicator).exists() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Detect programming languages in the project
    async fn detect_languages(&self, path: &Path) -> Result<Vec<String>> {
        let mut languages = Vec::new();

        // Check for language-specific files
        if path.join("Cargo.toml").exists() {
            languages.push("Rust".to_string());
        }
        if path.join("package.json").exists() {
            languages.push("JavaScript".to_string());
        }
        if path.join("tsconfig.json").exists() {
            languages.push("TypeScript".to_string());
        }
        if path.join("pyproject.toml").exists() || path.join("setup.py").exists() {
            languages.push("Python".to_string());
        }
        if path.join("go.mod").exists() {
            languages.push("Go".to_string());
        }
        if path.join("pom.xml").exists() {
            languages.push("Java".to_string());
        }

        // If no specific markers, try to detect from file extensions
        if languages.is_empty() {
            languages = self.detect_from_extensions(path).await?;
        }

        Ok(languages)
    }

    /// Detect languages from file extensions
    async fn detect_from_extensions(&self, path: &Path) -> Result<Vec<String>> {
        let mut language_counts: HashMap<String, usize> = HashMap::new();

        if let Ok(mut entries) = fs::read_dir(path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(ext) = entry.path().extension() {
                    let lang = match ext.to_str() {
                        Some("rs") => Some("Rust"),
                        Some("js") => Some("JavaScript"),
                        Some("ts") | Some("tsx") => Some("TypeScript"),
                        Some("py") => Some("Python"),
                        Some("go") => Some("Go"),
                        Some("java") => Some("Java"),
                        Some("c") | Some("h") => Some("C"),
                        Some("cpp") | Some("cc") | Some("cxx") => Some("C++"),
                        _ => None,
                    };

                    if let Some(lang) = lang {
                        *language_counts.entry(lang.to_string()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Return languages sorted by count (most used first)
        let mut languages: Vec<_> = language_counts.into_iter().collect();
        languages.sort_by(|a, b| b.1.cmp(&a.1));
        Ok(languages.into_iter().map(|(lang, _)| lang).collect())
    }

    /// Extract description from README
    async fn extract_description(&self, path: &Path) -> Result<String> {
        let readme_paths = ["README.md", "README.txt", "README"];

        for readme_name in readme_paths {
            let readme_path = path.join(readme_name);
            if readme_path.exists() {
                if let Ok(content) = fs::read_to_string(&readme_path).await {
                    // Extract first meaningful line (not empty, not heading marker)
                    for line in content.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with('#') {
                            return Ok(line.to_string());
                        }
                    }
                }
            }
        }

        Ok("No description available".to_string())
    }

    /// Determine project status
    async fn determine_status(&self, path: &Path) -> Result<ProjectStatus> {
        // Check for deprecation indicators
        let deprecated_indicators = ["DEPRECATED", "deprecated", ".deprecated"];
        for indicator in deprecated_indicators {
            if path.join(indicator).exists() {
                return Ok(ProjectStatus::Deprecated);
            }
        }

        // Check for archive indicators
        if path.join("ARCHIVED").exists() {
            return Ok(ProjectStatus::Archived);
        }

        // Check git activity (if available)
        if let Ok(Some(last_commit)) = self.get_last_commit(path).await {
            if let Ok(commit_time) = chrono::DateTime::parse_from_rfc3339(&last_commit) {
                let age = chrono::Utc::now().signed_duration_since(commit_time);

                if age.num_days() < 30 {
                    return Ok(ProjectStatus::Active);
                } else if age.num_days() < 180 {
                    return Ok(ProjectStatus::Maintenance);
                } else {
                    return Ok(ProjectStatus::Archived);
                }
            }
        }

        Ok(ProjectStatus::Unknown)
    }

    /// Get last commit timestamp
    async fn get_last_commit(&self, path: &Path) -> Result<Option<String>> {
        let git_dir = path.join(".git");
        if !git_dir.exists() {
            return Ok(None);
        }

        // Try to get last commit using git command
        let output = tokio::process::Command::new("git")
            .args(["-C", path.to_str().unwrap(), "log", "-1", "--format=%cI"])
            .output()
            .await;

        if let Ok(output) = output {
            if output.status.success() {
                let timestamp = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !timestamp.is_empty() {
                    return Ok(Some(timestamp));
                }
            }
        }

        Ok(None)
    }
}
