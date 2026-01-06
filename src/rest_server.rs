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
struct TodoResponse {
    id: i64,
    title: String,
    description: String,
    completed: bool,
}

#[derive(Deserialize)]
pub struct CreateTodoRequest {
    title: String,
    description: String,
}

#[derive(Deserialize)]
pub struct UpdateTodoRequest {
    title: Option<String>,
    description: Option<String>,
    completed: Option<bool>,
}

fn model_to_response(model: db::TodoModel) -> TodoResponse {
    TodoResponse {
        id: model.id,
        title: model.title,
        description: model.description,
        completed: model.completed,
    }
}

pub fn create_router(pool: SqlitePool) -> Router {
    let state = AppState { pool };

    Router::new()
        .route("/todos", post(create_todo))
        .route("/todos", get(list_todos))
        .route("/todos/:id", get(get_todo))
        .route("/todos/:id", put(update_todo))
        .route("/todos/:id", delete(delete_todo))
        .with_state(state)
}

async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<CreateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    let todo = db::create_todo(&state.pool, &payload.title, &payload.description).await?;
    Ok(Json(model_to_response(todo)))
}

async fn get_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<TodoResponse>, AppError> {
    let todo = db::get_todo(&state.pool, id).await?;
    Ok(Json(model_to_response(todo)))
}

async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<TodoResponse>>, AppError> {
    let todos = db::list_todos(&state.pool).await?;
    let todos = todos.into_iter().map(model_to_response).collect();
    Ok(Json(todos))
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTodoRequest>,
) -> Result<Json<TodoResponse>, AppError> {
    let todo = db::update_todo(
        &state.pool,
        id,
        payload.title.as_deref(),
        payload.description.as_deref(),
        payload.completed,
    )
    .await?;
    Ok(Json(model_to_response(todo)))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let success = db::delete_todo(&state.pool, id).await?;
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
                (StatusCode::NOT_FOUND, "Todo not found")
            }
            AppError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            AppError::NotFound => (StatusCode::NOT_FOUND, "Todo not found"),
        };

        (status, message).into_response()
    }
}
