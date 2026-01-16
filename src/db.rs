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

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UserModel {
    pub id: i64,
    pub name: String,
    pub email: String,
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

    // Create the users table if it doesn't exist
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE
        )
        "#,
    )
    .execute(&pool)
    .await?;

    Ok(pool)
}
