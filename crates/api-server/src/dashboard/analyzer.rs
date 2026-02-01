use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;
use tracing::debug;

use super::models::{Project, ProjectAnalysis, ProjectMetrics, ProjectStatus};

pub struct ProjectAnalyzer;

impl ProjectAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Calculate health score for a project
    pub async fn calculate_health_score(&self, project: &Project) -> Result<f64> {
        let path = Path::new(&project.path);
        let mut factors = HashMap::new();

        // Factor 1: Recent commit activity (0-30 points)
        let commit_score = self.calculate_commit_score(&project.last_commit).await;
        factors.insert("commit_activity".to_string(), commit_score);

        // Factor 2: Documentation (0-20 points)
        let docs_score = self.calculate_docs_score(path).await?;
        factors.insert("documentation".to_string(), docs_score);

        // Factor 3: Testing (0-20 points)
        let test_score = self.calculate_test_score(path).await?;
        factors.insert("testing".to_string(), test_score);

        // Factor 4: Code organization (0-15 points)
        let org_score = self.calculate_organization_score(path).await?;
        factors.insert("organization".to_string(), org_score);

        // Factor 5: Dependencies health (0-15 points)
        let deps_score = self.calculate_deps_score(path).await?;
        factors.insert("dependencies".to_string(), deps_score);

        // Total score (out of 100)
        let total_score = factors.values().sum::<f64>();

        debug!(
            "Health score for {}: {:.1} (factors: {:?})",
            project.name, total_score, factors
        );

        Ok(total_score)
    }

    /// Calculate commit activity score
    async fn calculate_commit_score(&self, last_commit: &Option<String>) -> f64 {
        if let Some(commit_time) = last_commit {
            if let Ok(commit_dt) = chrono::DateTime::parse_from_rfc3339(commit_time) {
                let age = chrono::Utc::now().signed_duration_since(commit_dt);
                let days = age.num_days();

                // Scoring: < 7 days = 30pts, < 30 days = 20pts, < 90 days = 10pts, else 0pts
                return if days < 7 {
                    30.0
                } else if days < 30 {
                    20.0
                } else if days < 90 {
                    10.0
                } else {
                    0.0
                };
            }
        }
        0.0
    }

    /// Calculate documentation score
    async fn calculate_docs_score(&self, path: &Path) -> Result<f64> {
        let mut score: f64 = 0.0;

        // Check for README (10 points)
        if path.join("README.md").exists() || path.join("README").exists() {
            score += 10.0;
        }

        // Check for additional documentation (5 points each, max 10)
        let doc_files = ["CHANGELOG.md", "CONTRIBUTING.md", "LICENSE", "docs/"];
        let doc_count = doc_files
            .iter()
            .filter(|&file| path.join(file).exists())
            .count();
        score += (doc_count as f64 * 5.0).min(10.0);

        Ok(score)
    }

    /// Calculate testing score
    async fn calculate_test_score(&self, path: &Path) -> Result<f64> {
        let mut score: f64 = 0.0;

        // Check for test directories/files
        let test_indicators = [
            "tests/",
            "test/",
            "__tests__/",
            "spec/",
            ".github/workflows/",
        ];

        for indicator in test_indicators {
            if path.join(indicator).exists() {
                score += 10.0;
                break;
            }
        }

        // Check for CI configuration (10 points)
        let ci_files = [
            ".github/workflows/ci.yml",
            ".github/workflows/test.yml",
            ".gitlab-ci.yml",
            ".travis.yml",
            "Jenkinsfile",
        ];
        if ci_files.iter().any(|file| path.join(file).exists()) {
            score += 10.0;
        }

        Ok(score.min(20.0))
    }

    /// Calculate code organization score
    async fn calculate_organization_score(&self, path: &Path) -> Result<f64> {
        let mut score: f64 = 0.0;

        // Check for standard project structure
        let structure_dirs = ["src/", "lib/", "pkg/", "cmd/"];
        if structure_dirs.iter().any(|dir| path.join(dir).exists()) {
            score += 8.0;
        }

        // Check for configuration files
        let config_files = [
            ".editorconfig",
            ".gitignore",
            ".rustfmt.toml",
            "prettier.config.js",
            "tsconfig.json",
        ];
        let config_count = config_files
            .iter()
            .filter(|&file| path.join(file).exists())
            .count();
        score += (config_count as f64 * 2.0).min(7.0);

        Ok(score)
    }

    /// Calculate dependencies health score
    async fn calculate_deps_score(&self, path: &Path) -> Result<f64> {
        let mut score: f64 = 15.0; // Start optimistic

        // Check for lock files (good sign)
        let lock_files = [
            "Cargo.lock",
            "package-lock.json",
            "yarn.lock",
            "poetry.lock",
        ];
        let has_lock = lock_files.iter().any(|file| path.join(file).exists());
        if !has_lock {
            score -= 5.0;
        }

        // Check for security audit files
        if path.join(".github/workflows/security.yml").exists() {
            score += 0.0; // Already at max
        }

        Ok(score)
    }

    /// Generate full project analysis
    pub async fn analyze_project(&self, mut project: Project) -> Result<ProjectAnalysis> {
        let health_score = self.calculate_health_score(&project).await?;
        project.health_score = health_score;

        let mut insights = Vec::new();
        let mut recommendations = Vec::new();

        // Generate insights based on health score
        if health_score >= 80.0 {
            insights.push("Project is in excellent health".to_string());
        } else if health_score >= 60.0 {
            insights.push("Project is generally healthy with room for improvement".to_string());
        } else if health_score >= 40.0 {
            insights.push("Project shows signs of technical debt".to_string());
            recommendations.push("Consider improving documentation and test coverage".to_string());
        } else {
            insights.push("Project requires significant attention".to_string());
            recommendations
                .push("Prioritize documentation, testing, and recent commits".to_string());
        }

        // Status-based insights
        match project.status {
            ProjectStatus::Active => {
                insights.push("Active development with recent commits".to_string());
            }
            ProjectStatus::Maintenance => {
                insights
                    .push("In maintenance mode - consider reviewing if still needed".to_string());
            }
            ProjectStatus::Deprecated => {
                insights.push("Marked as deprecated - plan migration or removal".to_string());
                recommendations.push("Document migration path for dependents".to_string());
            }
            ProjectStatus::Archived => {
                insights.push("Archived project - no recent activity".to_string());
            }
            ProjectStatus::Unknown => {
                insights.push("Status unclear - needs manual review".to_string());
            }
        }

        // Language-based insights
        if project.languages.is_empty() {
            insights.push("No primary language detected".to_string());
        }

        let analysis = ProjectAnalysis {
            project: project.name.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            health_score,
            status: project.status.clone(),
            insights,
            recommendations,
            metrics: ProjectMetrics {
                health_factors: HashMap::new(), // Could populate with detailed metrics
            },
        };

        Ok(analysis)
    }

    /// Calculate overall ecosystem health
    pub async fn calculate_ecosystem_health(&self, projects: &[Project]) -> f64 {
        if projects.is_empty() {
            return 0.0;
        }

        let total: f64 = projects.iter().map(|p| p.health_score).sum();
        total / projects.len() as f64
    }
}
