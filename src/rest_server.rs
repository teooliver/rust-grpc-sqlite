use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::db;
use crate::repository::{TaskRepository, UserRepository};

#[derive(Clone)]
pub struct AppState {
    task_repository: Arc<dyn TaskRepository>,
    user_repository: Arc<dyn UserRepository>,
}

#[derive(Serialize)]
struct TaskResponse {
    id: i64,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    title: String,
    description: String,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

// User DTOs

#[derive(Serialize)]
struct UserResponse {
    id: i64,
    name: String,
    email: String,
}

#[derive(Deserialize)]
pub struct CreateUserRequest {
    name: String,
    email: String,
}

#[derive(Deserialize)]
pub struct UpdateUserRequest {
    name: Option<String>,
    email: Option<String>,
}

fn model_to_response(model: db::TaskModel) -> TaskResponse {
    TaskResponse {
        id: model.id,
        title: model.title,
        description: model.description,
        completed: model.completed,
    }
}

fn user_model_to_response(model: db::UserModel) -> UserResponse {
    UserResponse {
        id: model.id,
        name: model.name,
        email: model.email,
    }
}

pub fn create_router(
    task_repository: Arc<dyn TaskRepository>,
    user_repository: Arc<dyn UserRepository>,
) -> Router {
    let state = AppState {
        task_repository,
        user_repository,
    };

    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        .route("/users", post(create_user))
        .route("/users", get(list_users))
        .route("/users/:id", get(get_user))
        .route("/users/:id", put(update_user))
        .route("/users/:id", delete(delete_user))
        .with_state(state)
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = state
        .task_repository
        .create(&payload.title, &payload.description)
        .await?;
    Ok(Json(model_to_response(task)))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = state.task_repository.get(id).await?;
    Ok(Json(model_to_response(task)))
}

async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<TaskResponse>>, AppError> {
    let tasks = state.task_repository.list().await?;
    let tasks = tasks.into_iter().map(model_to_response).collect();
    Ok(Json(tasks))
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = state
        .task_repository
        .update(
            id,
            payload.title.as_deref(),
            payload.description.as_deref(),
            payload.completed,
        )
        .await?;
    Ok(Json(model_to_response(task)))
}

async fn delete_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let success = state.task_repository.delete(id).await?;
    if success {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// User handlers

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state
        .user_repository
        .create(&payload.name, &payload.email)
        .await?;
    Ok(Json(user_model_to_response(user)))
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state.user_repository.get(id).await?;
    Ok(Json(user_model_to_response(user)))
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<UserResponse>>, AppError> {
    let users = state.user_repository.list().await?;
    let users = users.into_iter().map(user_model_to_response).collect();
    Ok(Json(users))
}

async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, AppError> {
    let user = state
        .user_repository
        .update(id, payload.name.as_deref(), payload.email.as_deref())
        .await?;
    Ok(Json(user_model_to_response(user)))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let success = state.user_repository.delete(id).await?;
    if success {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(AppError::NotFound)
    }
}

// Error handling
enum AppError {
    Database(sqlx::Error),
    NotFound,
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        // Try to downcast to sqlx::Error
        if let Some(sqlx_err) = err.downcast_ref::<sqlx::Error>() {
            match sqlx_err {
                sqlx::Error::RowNotFound => AppError::NotFound,
                _ => AppError::Database(sqlx::Error::Protocol(err.to_string())),
            }
        } else {
            AppError::Database(sqlx::Error::Protocol(err.to_string()))
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::Database(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "Task not found")
            }
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Task not found"),
        };

        (status, message).into_response()
    }
}
