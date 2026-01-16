use rust_grpc_sqlite::repository::{SqliteTaskRepository, SqliteUserRepository};
use sqlx::SqlitePool;
use std::sync::Arc;

pub async fn setup_test_pool() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

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
    .await
    .unwrap();

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
    .await
    .unwrap();

    pool
}

pub async fn setup_test_repository() -> Arc<SqliteTaskRepository> {
    let pool = setup_test_pool().await;
    Arc::new(SqliteTaskRepository::new(pool))
}

pub async fn setup_test_user_repository() -> Arc<SqliteUserRepository> {
    let pool = setup_test_pool().await;
    Arc::new(SqliteUserRepository::new(pool))
}

pub async fn setup_test_pool_with_data() -> SqlitePool {
    let pool = setup_test_pool().await;

    sqlx::query("INSERT INTO tasks (title, description, completed) VALUES (?, ?, ?)")
        .bind("Test Task 1")
        .bind("Description 1")
        .bind(false)
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO tasks (title, description, completed) VALUES (?, ?, ?)")
        .bind("Test Task 2")
        .bind("Description 2")
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

    pool
}

pub async fn setup_test_repository_with_data() -> Arc<SqliteTaskRepository> {
    let pool = setup_test_pool_with_data().await;
    Arc::new(SqliteTaskRepository::new(pool))
}

pub async fn setup_test_pool_with_user_data() -> SqlitePool {
    let pool = setup_test_pool().await;

    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("John Doe")
        .bind("john@example.com")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("INSERT INTO users (name, email) VALUES (?, ?)")
        .bind("Jane Doe")
        .bind("jane@example.com")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

pub async fn setup_test_user_repository_with_data() -> Arc<SqliteUserRepository> {
    let pool = setup_test_pool_with_user_data().await;
    Arc::new(SqliteUserRepository::new(pool))
}
