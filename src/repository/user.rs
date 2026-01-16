use anyhow::Result;
use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::db::UserModel;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, name: &str, email: &str) -> Result<UserModel>;
    async fn get(&self, id: i64) -> Result<UserModel>;
    async fn list(&self) -> Result<Vec<UserModel>>;
    async fn update(&self, id: i64, name: Option<&str>, email: Option<&str>) -> Result<UserModel>;
    async fn delete(&self, id: i64) -> Result<bool>;
}

#[derive(Clone)]
pub struct SqliteUserRepository {
    pool: SqlitePool,
}

impl SqliteUserRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for SqliteUserRepository {
    async fn create(&self, name: &str, email: &str) -> Result<UserModel> {
        let user = sqlx::query_as::<_, UserModel>(
            "INSERT INTO users (name, email) VALUES (?, ?) RETURNING *",
        )
        .bind(name)
        .bind(email)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn get(&self, id: i64) -> Result<UserModel> {
        let user = sqlx::query_as::<_, UserModel>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await?;

        Ok(user)
    }

    async fn list(&self) -> Result<Vec<UserModel>> {
        let users = sqlx::query_as::<_, UserModel>("SELECT * FROM users ORDER BY id DESC")
            .fetch_all(&self.pool)
            .await?;

        Ok(users)
    }

    async fn update(&self, id: i64, name: Option<&str>, email: Option<&str>) -> Result<UserModel> {
        let existing = self.get(id).await?;

        let new_name = name.unwrap_or(&existing.name);
        let new_email = email.unwrap_or(&existing.email);

        let user = sqlx::query_as::<_, UserModel>(
            "UPDATE users SET name = ?, email = ? WHERE id = ? RETURNING *",
        )
        .bind(new_name)
        .bind(new_email)
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(user)
    }

    async fn delete(&self, id: i64) -> Result<bool> {
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_repository() -> SqliteUserRepository {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

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

        SqliteUserRepository::new(pool)
    }

    #[tokio::test]
    async fn test_create_user() {
        let repo = setup_test_repository().await;

        let user = repo.create("John Doe", "john@example.com").await.unwrap();

        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert!(user.id > 0);
    }

    #[tokio::test]
    async fn test_get_user() {
        let repo = setup_test_repository().await;

        let created = repo.create("Jane Doe", "jane@example.com").await.unwrap();
        let retrieved = repo.get(created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Jane Doe");
        assert_eq!(retrieved.email, "jane@example.com");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let repo = setup_test_repository().await;

        let result = repo.get(999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_users() {
        let repo = setup_test_repository().await;

        let user1 = repo.create("User 1", "user1@example.com").await.unwrap();
        let user2 = repo.create("User 2", "user2@example.com").await.unwrap();

        let users = repo.list().await.unwrap();

        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user2.id);
        assert_eq!(users[1].id, user1.id);
    }

    #[tokio::test]
    async fn test_update_user() {
        let repo = setup_test_repository().await;

        let user = repo
            .create("Original Name", "original@example.com")
            .await
            .unwrap();
        let updated = repo
            .update(user.id, Some("Updated Name"), None)
            .await
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.email, "original@example.com");
    }

    #[tokio::test]
    async fn test_delete_user() {
        let repo = setup_test_repository().await;

        let user = repo
            .create("Delete Me", "delete@example.com")
            .await
            .unwrap();
        let deleted = repo.delete(user.id).await.unwrap();

        assert_eq!(deleted, true);

        let result = repo.get(user.id).await;
        assert!(result.is_err());
    }
}
