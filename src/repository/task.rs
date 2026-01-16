use anyhow::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::db::TaskModel;

#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create(&self, title: &str, description: &str) -> Result<TaskModel>;
    async fn get(&self, id: i64) -> Result<TaskModel>;
    async fn list(&self) -> Result<Vec<TaskModel>>;
    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<&str>,
        completed: Option<bool>,
    ) -> Result<TaskModel>;
    async fn delete(&self, id: i64) -> Result<bool>;
}

#[derive(Clone)]
pub struct SqliteTaskRepository {
    pool: SqlitePool,
}

impl SqliteTaskRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TaskRepository for SqliteTaskRepository {
    async fn create(&self, title: &str, description: &str) -> Result<TaskModel> {
        let task = sqlx::query_as::<_, TaskModel>(
            "INSERT INTO tasks (title, description, completed) VALUES (?, ?, 0) RETURNING *",
        )
        .bind(title)
        .bind(description)
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    async fn get(&self, id: i64) -> Result<TaskModel> {
        let task = sqlx::query_as::<_, TaskModel>("SELECT * FROM tasks WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(task)
    }

    async fn list(&self) -> Result<Vec<TaskModel>> {
        let tasks = sqlx::query_as::<_, TaskModel>("SELECT * FROM tasks ORDER BY id DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(tasks)
    }

    async fn update(
        &self,
        id: i64,
        title: Option<&str>,
        description: Option<&str>,
        completed: Option<bool>,
    ) -> Result<TaskModel> {
        let existing = self.get(id).await?;

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
        .fetch_one(&self.pool)
        .await?;

        Ok(task)
    }

    async fn delete(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_repository() -> SqliteTaskRepository {
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

        SqliteTaskRepository::new(pool)
    }

    #[tokio::test]
    async fn test_create_task() {
        let repo = setup_test_repository().await;

        let task = repo.create("Test Task", "Test Description").await.unwrap();

        assert_eq!(task.title, "Test Task");
        assert_eq!(task.description, "Test Description");
        assert_eq!(task.completed, false);
        assert!(task.id > 0);
    }

    #[tokio::test]
    async fn test_get_task() {
        let repo = setup_test_repository().await;

        let created = repo.create("Find Me", "Description").await.unwrap();
        let retrieved = repo.get(created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.title, "Find Me");
    }

    #[tokio::test]
    async fn test_get_task_not_found() {
        let repo = setup_test_repository().await;

        let result = repo.get(999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_tasks() {
        let repo = setup_test_repository().await;

        let task1 = repo.create("Task 1", "Desc 1").await.unwrap();
        let task2 = repo.create("Task 2", "Desc 2").await.unwrap();

        let tasks = repo.list().await.unwrap();

        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].id, task2.id);
        assert_eq!(tasks[1].id, task1.id);
    }

    #[tokio::test]
    async fn test_update_task() {
        let repo = setup_test_repository().await;

        let task = repo.create("Original", "Original Desc").await.unwrap();
        let updated = repo
            .update(task.id, Some("Updated"), None, Some(true))
            .await
            .unwrap();

        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.description, "Original Desc");
        assert_eq!(updated.completed, true);
    }

    #[tokio::test]
    async fn test_delete_task() {
        let repo = setup_test_repository().await;

        let task = repo.create("Delete Me", "Description").await.unwrap();
        let deleted = repo.delete(task.id).await.unwrap();

        assert_eq!(deleted, true);

        let result = repo.get(task.id).await;
        assert!(result.is_err());
    }
}
