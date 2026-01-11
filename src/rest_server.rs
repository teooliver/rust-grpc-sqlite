use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::db;

#[derive(Clone)]
pub struct AppState {
    pool: SqlitePool,
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

fn model_to_response(model: db::TaskModel) -> TaskResponse {
    TaskResponse {
        id: model.id,
        title: model.title,
        description: model.description,
        completed: model.completed,
    }
}

pub fn create_router(pool: SqlitePool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/tasks", post(create_task))
        .route("/tasks", get(list_tasks))
        .route("/tasks/:id", get(get_task))
        .route("/tasks/:id", put(update_task))
        .route("/tasks/:id", delete(delete_task))
        .with_state(state)
}

async fn create_task(
    State(state): State<AppState>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::create_task(&state.pool, &payload.title, &payload.description).await?;
    Ok(Json(model_to_response(task)))
}

async fn get_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::get_task(&state.pool, id).await?;
    Ok(Json(model_to_response(task)))
}

async fn list_tasks(State(state): State<AppState>) -> Result<Json<Vec<TaskResponse>>, AppError> {
    let tasks = db::list_tasks(&state.pool).await?;
    let tasks = tasks.into_iter().map(model_to_response).collect();
    Ok(Json(tasks))
}

async fn update_task(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, AppError> {
    let task = db::update_task(
        &state.pool,
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
    let success = db::delete_task(&state.pool, id).await?;
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
    fn from(_err: anyhow::Error) -> Self {
        AppError::Database(sqlx::Error::Protocol("Unknown error".to_string()))
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
