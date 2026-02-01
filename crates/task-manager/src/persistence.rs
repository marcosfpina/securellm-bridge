//! SQLite persistence for tasks

use crate::Task;
use anyhow::Result;
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

pub struct TaskStore {
    pool: SqlitePool,
}

impl TaskStore {
    pub async fn new(db_path: &str) -> Result<Self> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&format!("sqlite:{}", db_path))
            .await?;

        // Create schema
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tasks (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                priority INTEGER NOT NULL,
                state TEXT NOT NULL,
                progress REAL NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                metadata TEXT
            )
            "#,
        )
        .execute(&pool)
        .await?;

        Ok(Self { pool })
    }

    pub async fn save_task(&self, task: &Task) -> Result<()> {
        let state_json = serde_json::to_string(&task.state)?;
        let metadata_json = serde_json::to_string(&task.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO tasks (id, name, description, priority, state, progress, created_at, updated_at, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                description = excluded.description,
                priority = excluded.priority,
                state = excluded.state,
                progress = excluded.progress,
                updated_at = excluded.updated_at,
                metadata = excluded.metadata
            "#,
        )
        .bind(task.id.to_string())
        .bind(&task.name)
        .bind(&task.description)
        .bind(task.priority as i64)
        .bind(state_json)
        .bind(task.progress as f64)
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn load_task(&self, id: &str) -> Result<Option<Task>> {
        let row: Option<(String, String, Option<String>, i64, String, f64, String, String, String)> =
            sqlx::query_as(
                "SELECT id, name, description, priority, state, progress, created_at, updated_at, metadata FROM tasks WHERE id = ?"
            )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((
            id,
            name,
            description,
            priority,
            state_json,
            progress,
            created_at,
            updated_at,
            metadata_json,
        )) = row
        {
            Ok(Some(Task {
                id: uuid::Uuid::parse_str(&id)?,
                name,
                description,
                priority: priority as u8,
                state: serde_json::from_str(&state_json)?,
                progress: progress as f32,
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?
                    .with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)?
                    .with_timezone(&chrono::Utc),
                metadata: serde_json::from_str(&metadata_json)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn load_all_tasks(&self) -> Result<Vec<Task>> {
        let rows: Vec<(String, String, Option<String>, i64, String, f64, String, String, String)> =
            sqlx::query_as(
                "SELECT id, name, description, priority, state, progress, created_at, updated_at, metadata FROM tasks ORDER BY created_at DESC"
            )
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(
                |(
                    id,
                    name,
                    description,
                    priority,
                    state_json,
                    progress,
                    created_at,
                    updated_at,
                    metadata_json,
                )| {
                    Ok(Task {
                        id: uuid::Uuid::parse_str(&id)?,
                        name,
                        description,
                        priority: priority as u8,
                        state: serde_json::from_str(&state_json)?,
                        progress: progress as f32,
                        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?
                            .with_timezone(&chrono::Utc),
                        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)?
                            .with_timezone(&chrono::Utc),
                        metadata: serde_json::from_str(&metadata_json)?,
                    })
                },
            )
            .collect()
    }
}
