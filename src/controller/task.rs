use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::TaskModel;

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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
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

    #[tokio::test]
    async fn test_create_task() {
        let pool = setup_test_db().await;

        let task = create_task(&pool, "Test Task", "Test Description")
            .await
            .unwrap();

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Test Description");
        assert_eq!(task.completed, false);
        assert!(task.id > 0);
    }

    #[tokio::test]
    async fn test_create_task_sets_completed_false() {
        let pool = setup_test_db().await;

        let task = create_task(&pool, "New Task", "New Description")
            .await
            .unwrap();

        assert_eq!(task.completed, false);
    }

    #[tokio::test]
    async fn test_get_task() {
        let pool = setup_test_db().await;

        let created = create_task(&pool, "Find Me", "Description").await.unwrap();

        let retrieved = get_task(&pool, created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.title, "Find Me");
        assert_eq!(retrieved.description, "Description");
        assert_eq!(retrieved.completed, false);
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let pool = setup_test_db().await;

        let result = get_task(&pool, 999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tasks_empty() {
        let pool = setup_test_db().await;

        let tasks = list_tasks(&pool).await.unwrap();

        assert_eq!(tasks.len(), 0);
    }

    #[tokio::test]
    async fn test_list_tasks_multiple() {
        let pool = setup_test_db().await;

        let task1 = create_task(&pool, "Task 1", "Description 1").await.unwrap();
        let task2 = create_task(&pool, "Task 2", "Description 2").await.unwrap();
        let task3 = create_task(&pool, "Task 3", "Description 3").await.unwrap();

        let tasks = list_tasks(&pool).await.unwrap();

        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].id, task3.id);
        assert_eq!(tasks[1].id, task2.id);
        assert_eq!(tasks[2].id, task1.id);
    }

    #[tokio::test]
    async fn test_update_task_all_fields() {
        let pool = setup_test_db().await;

        let task = create_task(&pool, "Original", "Original Description")
            .await
            .unwrap();

        let updated = update_task(
            &pool,
            task.id,
            Some("Updated"),
            Some("Updated Description"),
            Some(true),
        )
        .await
        .unwrap();

        assert_eq!(updated.id, task.id);
        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.description, "Updated Description");
        assert_eq!(updated.completed, true);
    }

    #[tokio::test]
    async fn test_update_task_partial_title_only() {
        let pool = setup_test_db().await;

        let task = create_task(&pool, "Original", "Original Description")
            .await
            .unwrap();

        let updated = update_task(&pool, task.id, Some("New Title"), None, None)
            .await
            .unwrap();

        assert_eq!(updated.title, "New Title");
        assert_eq!(updated.description, "Original Description");
        assert_eq!(updated.completed, false);
    }

    #[tokio::test]
    async fn test_update_task_partial_completed_only() {
        let pool = setup_test_db().await;

        let task = create_task(&pool, "Task", "Description").await.unwrap();

        let updated = update_task(&pool, task.id, None, None, Some(true))
            .await
            .unwrap();

        assert_eq!(updated.title, "Task");
        assert_eq!(updated.description, "Description");
        assert_eq!(updated.completed, true);
    }

    #[tokio::test]
    async fn test_update_task_not_found() {
        let pool = setup_test_db().await;

        let result = update_task(&pool, 999, Some("Title"), None, None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_task() {
        let pool = setup_test_db().await;

        let task = create_task(&pool, "Delete Me", "Description")
            .await
            .unwrap();

        let deleted = delete_task(&pool, task.id).await.unwrap();

        assert_eq!(deleted, true);

        let result = get_task(&pool, task.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_task_not_found() {
        let pool = setup_test_db().await;

        let deleted = delete_task(&pool, 999).await.unwrap();

        assert_eq!(deleted, false);
    }

    #[tokio::test]
    async fn test_multiple_tasks_independence() {
        let pool = setup_test_db().await;

        let task1 = create_task(&pool, "Task 1", "Desc 1").await.unwrap();
        let task2 = create_task(&pool, "Task 2", "Desc 2").await.unwrap();

        update_task(&pool, task1.id, None, None, Some(true))
            .await
            .unwrap();

        let retrieved1 = get_task(&pool, task1.id).await.unwrap();
        let retrieved2 = get_task(&pool, task2.id).await.unwrap();

        assert_eq!(retrieved1.completed, true);
        assert_eq!(retrieved2.completed, false);
    }
}
