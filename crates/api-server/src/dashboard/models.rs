use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IntelligenceType {
    Sigint,
    Humint,
    Osint,
    Techint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThreatLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Maintenance,
    Deprecated,
    Archived,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: String,
    pub description: String,
    pub languages: Vec<String>,
    pub status: ProjectStatus,
    pub health_score: f64,
    pub last_commit: Option<String>,
    pub last_indexed: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    pub project: String,
    pub timestamp: String,
    pub health_score: f64,
    pub status: ProjectStatus,
    pub insights: Vec<String>,
    pub recommendations: Vec<String>,
    pub metrics: ProjectMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetrics {
    pub health_factors: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceItem {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: IntelligenceType,
    pub source: String,
    pub title: String,
    pub content: String,
    pub threat_level: ThreatLevel,
    pub timestamp: String,
    pub tags: Vec<String>,
    pub related_projects: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatus {
    pub total_projects: usize,
    pub active_projects: usize,
    pub health_score: f64,
    pub total_intelligence: usize,
    pub alerts_count: usize,
    pub last_scan: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    #[serde(rename = "type")]
    pub alert_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Briefing {
    #[serde(rename = "type")]
    pub briefing_type: String,
    pub classification: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ecosystem_status: Option<EcosystemStatusBrief>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_developments: Option<Vec<Development>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alerts: Option<Vec<Alert>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_summaries: Option<Vec<ProjectSummary>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemStatusBrief {
    pub total_projects: usize,
    pub active_projects: usize,
    pub health_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Development {
    #[serde(rename = "type")]
    pub dev_type: String,
    pub title: String,
    pub source: String,
    pub threat_level: ThreatLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub name: String,
    pub health_score: f64,
    pub status: ProjectStatus,
    pub insights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub query: String,
    pub results: Vec<IntelligenceItem>,
    pub total: usize,
    pub search_type: String, // "semantic" | "keyword"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceStats {
    pub total: usize,
    pub by_type: HashMap<String, usize>,
    pub by_threat: HashMap<String, usize>,
}
