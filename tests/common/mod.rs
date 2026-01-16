use sqlx::SqlitePool;

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

    pool
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
