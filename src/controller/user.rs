use anyhow::Result;
use sqlx::SqlitePool;

use crate::db::UserModel;

pub async fn create_user(pool: &SqlitePool, name: &str, email: &str) -> Result<UserModel> {
    let user =
        sqlx::query_as::<_, UserModel>("INSERT INTO users (name, email) VALUES (?, ?) RETURNING *")
            .bind(name)
            .bind(email)
            .fetch_one(pool)
            .await?;

    Ok(user)
}

pub async fn get_user(pool: &SqlitePool, id: i64) -> Result<UserModel> {
    let user = sqlx::query_as::<_, UserModel>("SELECT * FROM users WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await?;

    Ok(user)
}

pub async fn list_users(pool: &SqlitePool) -> Result<Vec<UserModel>> {
    let users = sqlx::query_as::<_, UserModel>("SELECT * FROM users ORDER BY id DESC")
        .fetch_all(pool)
        .await?;

    Ok(users)
}

pub async fn update_user(
    pool: &SqlitePool,
    id: i64,
    name: Option<&str>,
    email: Option<&str>,
) -> Result<UserModel> {
    // Get existing user first
    let existing = get_user(pool, id).await?;

    let new_name = name.unwrap_or(&existing.name);
    let new_email = email.unwrap_or(&existing.email);

    let user = sqlx::query_as::<_, UserModel>(
        "UPDATE users SET name = ?, email = ? WHERE id = ? RETURNING *",
    )
    .bind(new_name)
    .bind(new_email)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(user)
}

pub async fn delete_user(pool: &SqlitePool, id: i64) -> Result<bool> {
    let result = sqlx::query("DELETE FROM users WHERE id = ?")
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

    #[tokio::test]
    async fn test_create_user() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "John Doe", "john@example.com")
            .await
            .unwrap();

        assert_eq!(user.name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert!(user.id > 0);
    }

    #[tokio::test]
    async fn test_get_user() {
        let pool = setup_test_db().await;

        let created = create_user(&pool, "Jane Doe", "jane@example.com")
            .await
            .unwrap();

        let retrieved = get_user(&pool, created.id).await.unwrap();

        assert_eq!(retrieved.id, created.id);
        assert_eq!(retrieved.name, "Jane Doe");
        assert_eq!(retrieved.email, "jane@example.com");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let pool = setup_test_db().await;

        let result = get_user(&pool, 999).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_list_users_empty() {
        let pool = setup_test_db().await;

        let users = list_users(&pool).await.unwrap();

        assert_eq!(users.len(), 0);
    }

    #[tokio::test]
    async fn test_list_users_multiple() {
        let pool = setup_test_db().await;

        let user1 = create_user(&pool, "User 1", "user1@example.com")
            .await
            .unwrap();
        let user2 = create_user(&pool, "User 2", "user2@example.com")
            .await
            .unwrap();
        let user3 = create_user(&pool, "User 3", "user3@example.com")
            .await
            .unwrap();

        let users = list_users(&pool).await.unwrap();

        assert_eq!(users.len(), 3);
        assert_eq!(users[0].id, user3.id);
        assert_eq!(users[1].id, user2.id);
        assert_eq!(users[2].id, user1.id);
    }

    #[tokio::test]
    async fn test_update_user_all_fields() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "Original Name", "original@example.com")
            .await
            .unwrap();

        let updated = update_user(
            &pool,
            user.id,
            Some("Updated Name"),
            Some("updated@example.com"),
        )
        .await
        .unwrap();

        assert_eq!(updated.id, user.id);
        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.email, "updated@example.com");
    }

    #[tokio::test]
    async fn test_update_user_partial_name_only() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "Original Name", "original@example.com")
            .await
            .unwrap();

        let updated = update_user(&pool, user.id, Some("New Name"), None)
            .await
            .unwrap();

        assert_eq!(updated.name, "New Name");
        assert_eq!(updated.email, "original@example.com");
    }

    #[tokio::test]
    async fn test_update_user_partial_email_only() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "Name", "old@example.com").await.unwrap();

        let updated = update_user(&pool, user.id, None, Some("new@example.com"))
            .await
            .unwrap();

        assert_eq!(updated.name, "Name");
        assert_eq!(updated.email, "new@example.com");
    }

    #[tokio::test]
    async fn test_update_user_not_found() {
        let pool = setup_test_db().await;

        let result = update_user(&pool, 999, Some("Name"), None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_user() {
        let pool = setup_test_db().await;

        let user = create_user(&pool, "Delete Me", "delete@example.com")
            .await
            .unwrap();

        let deleted = delete_user(&pool, user.id).await.unwrap();

        assert_eq!(deleted, true);

        let result = get_user(&pool, user.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_user_not_found() {
        let pool = setup_test_db().await;

        let deleted = delete_user(&pool, 999).await.unwrap();

        assert_eq!(deleted, false);
    }
}
