use anyhow::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::analyzer::ProjectAnalyzer;
use super::models::Project;
use super::scanner::ProjectScanner;

#[derive(Clone)]
pub struct AppState {
    pub scanner: Arc<ProjectScanner>,
    pub analyzer: Arc<ProjectAnalyzer>,
    pub projects: Arc<RwLock<Vec<Project>>>,
    pub last_scan: Arc<RwLock<Option<String>>>,
}

impl AppState {
    pub async fn new() -> Result<Self> {
        // Default to ~/arch directory
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/kernelcore".to_string());
        let base_path = PathBuf::from(home).join("arch");

        let scanner = Arc::new(ProjectScanner::new(base_path));
        let analyzer = Arc::new(ProjectAnalyzer::new());
        let projects = Arc::new(RwLock::new(Vec::new()));
        let last_scan = Arc::new(RwLock::new(None));

        let state = Self {
            scanner,
            analyzer,
            projects,
            last_scan,
        };

        // Perform initial scan
        state.scan_projects().await?;

        Ok(state)
    }

    /// Scan for projects and update cache
    pub async fn scan_projects(&self) -> Result<()> {
        let mut scanned_projects = self.scanner.scan().await?;

        // Calculate health scores
        for project in &mut scanned_projects {
            let health_score = self.analyzer.calculate_health_score(project).await?;
            project.health_score = health_score;
        }

        // Update cache
        let mut projects = self.projects.write().await;
        *projects = scanned_projects;

        let mut last_scan = self.last_scan.write().await;
        *last_scan = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Get all projects
    pub async fn get_projects(&self) -> Vec<Project> {
        self.projects.read().await.clone()
    }

    /// Get a specific project by name
    pub async fn get_project(&self, name: &str) -> Option<Project> {
        let projects = self.projects.read().await;
        projects.iter().find(|p| p.name == name).cloned()
    }

    /// Get last scan time
    pub async fn get_last_scan(&self) -> Option<String> {
        self.last_scan.read().await.clone()
    }
}
