use anyhow::Result;
use sqlx::{
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
    SqlitePool,
};
use std::str::FromStr;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TodoModel {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

pub async fn init_db() -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str("sqlite://todos.db")?.create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    // Create the todos table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS todos (
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

pub async fn create_todo(pool: &SqlitePool, title: &str, description: &str) -> Result<TodoModel> {
    let todo = sqlx::query_as::<_, TodoModel>(
        "INSERT INTO todos (title, description, completed) VALUES (?, ?, 0) RETURNING *",
    )
    .bind(title)
    .bind(description)
    .fetch_one(pool)
    .await?;

    Ok(todo)
}

pub async fn get_todo(pool: &SqlitePool, id: i64) -> Result<TodoModel> {
    let todo = sqlx::query_as::<_, TodoModel>("SELECT * FROM todos WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(todo)
}

pub async fn list_todos(pool: &SqlitePool) -> Result<Vec<TodoModel>> {
    let todos = sqlx::query_as::<_, TodoModel>("SELECT * FROM todos ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    Ok(todos)
}

pub async fn update_todo(
    pool: &SqlitePool,
    id: i64,
    title: Option<&str>,
    description: Option<&str>,
    completed: Option<bool>,
) -> Result<TodoModel> {
    // Get existing todo first
    let existing = get_todo(pool, id).await?;

    let new_title = title.unwrap_or(&existing.title);
    let new_description = description.unwrap_or(&existing.description);
    let new_completed = completed.unwrap_or(existing.completed);

    let todo = sqlx::query_as::<_, TodoModel>(
        "UPDATE todos SET title = ?, description = ?, completed = ? WHERE id = ? RETURNING *",
    )
    .bind(new_title)
    .bind(new_description)
    .bind(new_completed)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(todo)
}

pub async fn delete_todo(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM todos WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
