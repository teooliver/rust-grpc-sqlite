use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TaskModel {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

pub async fn init_db() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite://tasks.db")?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Create the tasks table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            description TEXT NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}

pub async fn create_task(pool: &SqlitePool, title: &str, description: &str) -> Result<TaskModel> {
    let task = sqlx::query_as::<_, TaskModel>(
        "INSERT INTO tasks (title, description, completed) VALUES (?, ?, 0) RETURNING *",
    )
    .bind(title)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

pub async fn get_task(pool: &SqlitePool, id: i64) -> Result<TaskModel> {
    let task = sqlx::query_as::<_, TaskModel>("SELECT * FROM tasks WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(task)
}

pub async fn list_tasks(pool: &SqlitePool) -> Result<Vec<TaskModel>> {
    let tasks = sqlx::query_as::<_, TaskModel>("SELECT * FROM tasks ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    Ok(tasks)
}

pub async fn update_task(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    description: Option<&str>,
    completed: Option<bool>,
) -> Result<TaskModel> {
    // Get existing task first
    let existing = get_task(pool, id).await?;

    let new_title = title.unwrap_or(&existing.title);
    let new_description = description.unwrap_or(&existing.description);
    let new_completed = completed.unwrap_or(existing.completed);

    let task = sqlx::query_as::<_, TaskModel>(
        "UPDATE tasks SET title = ?, description = ?, completed = ? WHERE id = ? RETURNING *",
    )
    .bind(new_title)
    .bind(new_description)
    .bind(new_completed)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(task)
}

pub async fn delete_task(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
