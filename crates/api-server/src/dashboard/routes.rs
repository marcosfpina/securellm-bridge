use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::info;

use super::models::*;
use super::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/status", get(get_status))
        .route("/projects", get(get_projects))
        .route("/projects/:name", get(get_project))
        .route("/projects/:name/summarize", post(summarize_project))
        .route("/intelligence/query", get(query_intelligence))
        .route("/intelligence/stats", get(get_intelligence_stats))
        .route("/briefing/daily", get(get_daily_briefing))
        .route("/briefing/executive", get(get_executive_briefing))
        .route("/alerts", get(get_alerts))
        .route("/graph/dependencies", get(get_dependency_graph))
        .route("/scan", post(trigger_scan))
}

// GET /api/status
async fn get_status(State(state): State<AppState>) -> impl IntoResponse {
    let projects = state.get_projects().await;
    let active_count = projects
        .iter()
        .filter(|p| matches!(p.status, ProjectStatus::Active))
        .count();

    let ecosystem_health = state.analyzer.calculate_ecosystem_health(&projects).await;

    let status = EcosystemStatus {
        total_projects: projects.len(),
        active_projects: active_count,
        health_score: ecosystem_health,
        total_intelligence: 0, // TODO: Implement intelligence system
        alerts_count: projects.iter().filter(|p| p.health_score < 50.0).count(),
        last_scan: state.get_last_scan().await,
    };

    Json(status)
}

#[derive(Debug, Deserialize)]
struct ProjectsQuery {
    status: Option<String>,
    language: Option<String>,
    sort_by: Option<String>,
    order: Option<String>,
}

// GET /api/projects
async fn get_projects(
    State(state): State<AppState>,
    Query(params): Query<ProjectsQuery>,
) -> impl IntoResponse {
    let mut projects = state.get_projects().await;

    // Filter by status
    if let Some(status_filter) = params.status {
        projects
            .retain(|p| format!("{:?}", p.status).to_lowercase() == status_filter.to_lowercase());
    }

    // Filter by language
    if let Some(lang_filter) = params.language {
        projects.retain(|p| {
            p.languages
                .iter()
                .any(|l| l.to_lowercase() == lang_filter.to_lowercase())
        });
    }

    // Sort
    match params.sort_by.as_deref() {
        Some("health_score") => {
            projects.sort_by(|a, b| {
                if params.order.as_deref() == Some("desc") {
                    b.health_score.partial_cmp(&a.health_score).unwrap()
                } else {
                    a.health_score.partial_cmp(&b.health_score).unwrap()
                }
            });
        }
        Some("name") => {
            projects.sort_by(|a, b| {
                if params.order.as_deref() == Some("desc") {
                    b.name.cmp(&a.name)
                } else {
                    a.name.cmp(&b.name)
                }
            });
        }
        _ => {}
    }

    Json(projects)
}

// GET /api/projects/:name
async fn get_project(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    if let Some(project) = state.get_project(&name).await {
        Ok(Json(project))
    } else {
        Err((StatusCode::NOT_FOUND, "Project not found"))
    }
}

// POST /api/projects/:name/summarize
async fn summarize_project(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    if let Some(project) = state.get_project(&name).await {
        match state.analyzer.analyze_project(project).await {
            Ok(analysis) => Ok(Json(analysis)),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Analysis failed: {}", e),
            )),
        }
    } else {
        Err((StatusCode::NOT_FOUND, "Project not found".to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct IntelligenceQuery {
    q: String,
    #[serde(default)]
    types: Option<String>,
    #[serde(default)]
    projects: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    semantic: Option<bool>,
}

// GET /api/intelligence/query
async fn query_intelligence(Query(_params): Query<IntelligenceQuery>) -> impl IntoResponse {
    // TODO: Implement intelligence search
    let result = QueryResult {
        query: _params.q,
        results: vec![],
        total: 0,
        search_type: if _params.semantic.unwrap_or(false) {
            "semantic".to_string()
        } else {
            "keyword".to_string()
        },
    };

    Json(result)
}

// GET /api/intelligence/stats
async fn get_intelligence_stats() -> impl IntoResponse {
    // TODO: Implement intelligence stats
    let stats = IntelligenceStats {
        total: 0,
        by_type: HashMap::new(),
        by_threat: HashMap::new(),
    };

    Json(stats)
}

// GET /api/briefing/daily
async fn get_daily_briefing(State(state): State<AppState>) -> impl IntoResponse {
    let projects = state.get_projects().await;
    let ecosystem_health = state.analyzer.calculate_ecosystem_health(&projects).await;

    let active_count = projects
        .iter()
        .filter(|p| matches!(p.status, ProjectStatus::Active))
        .count();

    // Get projects needing attention (health < 50)
    let troubled_projects: Vec<_> = projects
        .iter()
        .filter(|p| p.health_score < 50.0)
        .take(5)
        .collect();

    let summary = format!(
        "Ecosystem Overview: {} total projects, {} active. Overall health score: {:.1}%. {} projects require attention.",
        projects.len(),
        active_count,
        ecosystem_health,
        troubled_projects.len()
    );

    let briefing = Briefing {
        briefing_type: "daily".to_string(),
        classification: "UNCLASSIFIED".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary: Some(summary),
        headline: Some("Daily Intelligence Briefing".to_string()),
        ecosystem_status: Some(EcosystemStatusBrief {
            total_projects: projects.len(),
            active_projects: active_count,
            health_score: ecosystem_health,
        }),
        key_developments: Some(vec![]),
        alerts: Some(
            troubled_projects
                .iter()
                .map(|p| Alert {
                    alert_type: "health".to_string(),
                    project: Some(p.name.clone()),
                    message: format!(
                        "Project {} has low health score: {:.1}%",
                        p.name, p.health_score
                    ),
                    score: Some(p.health_score),
                })
                .collect(),
        ),
        project_summaries: Some(
            troubled_projects
                .iter()
                .map(|p| ProjectSummary {
                    name: p.name.clone(),
                    health_score: p.health_score,
                    status: p.status.clone(),
                    insights: vec![format!("Needs attention - health score below threshold")],
                })
                .collect(),
        ),
    };

    Json(briefing)
}

// GET /api/briefing/executive
async fn get_executive_briefing(State(state): State<AppState>) -> impl IntoResponse {
    let projects = state.get_projects().await;
    let ecosystem_health = state.analyzer.calculate_ecosystem_health(&projects).await;

    let active_count = projects
        .iter()
        .filter(|p| matches!(p.status, ProjectStatus::Active))
        .count();

    let summary = format!(
        "Executive Summary: The ~/arch ecosystem comprises {} projects with an overall health score of {:.1}%. {} projects are actively developed.",
        projects.len(),
        ecosystem_health,
        active_count
    );

    let briefing = Briefing {
        briefing_type: "executive".to_string(),
        classification: "UNCLASSIFIED".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary: Some(summary),
        headline: Some("Executive Intelligence Briefing".to_string()),
        ecosystem_status: Some(EcosystemStatusBrief {
            total_projects: projects.len(),
            active_projects: active_count,
            health_score: ecosystem_health,
        }),
        key_developments: None,
        alerts: None,
        project_summaries: None,
    };

    Json(briefing)
}

// GET /api/alerts
async fn get_alerts(State(state): State<AppState>) -> impl IntoResponse {
    let projects = state.get_projects().await;

    let alerts: Vec<Alert> = projects
        .iter()
        .filter(|p| p.health_score < 50.0)
        .map(|p| Alert {
            alert_type: "health".to_string(),
            project: Some(p.name.clone()),
            message: format!("Low health score: {:.1}%", p.health_score),
            score: Some(p.health_score),
        })
        .collect();

    Json(alerts)
}

// GET /api/graph/dependencies
async fn get_dependency_graph() -> impl IntoResponse {
    // TODO: Implement dependency graph analysis
    let graph = DependencyGraph {
        nodes: vec![],
        edges: vec![],
    };

    Json(graph)
}

// POST /api/scan
async fn trigger_scan(State(state): State<AppState>) -> impl IntoResponse {
    info!("🔄 Manual scan triggered");

    match state.scan_projects().await {
        Ok(_) => Json(serde_json::json!({
            "message": "Scan completed successfully"
        })),
        Err(e) => {
            tracing::error!("Scan failed: {}", e);
            Json(serde_json::json!({
                "message": format!("Scan failed: {}", e)
            }))
        }
    }
}
