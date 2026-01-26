use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::db::TaskModel;
use crate::repository::TaskRepository;

use super::{CreateTaskRequest, ErrorResponse, TaskResponse, UpdateTaskRequest};

pub fn task_routes<R: TaskRepository + 'static>(repo: Arc<R>) -> Router {
    Router::new()
        .route("/tasks", get(list_tasks::<R>).post(create_task::<R>))
        .route(
            "/tasks/{id}",
            get(get_task::<R>)
                .put(update_task::<R>)
                .delete(delete_task::<R>),
        )
        .with_state(repo)
}

impl From<TaskModel> for TaskResponse {
    fn from(model: TaskModel) -> Self {
        TaskResponse {
            id: model.id,
            title: model.title,
            description: model.description,
            completed: model.completed,
        }
    }
}

/// List all tasks
#[utoipa::path(
    get,
    path = "/api/tasks",
    responses(
        (status = 200, description = "List of all tasks", body = Vec<TaskResponse>),
    ),
    tag = "tasks"
)]
pub async fn list_tasks<R: TaskRepository>(
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<TaskResponse>>, impl IntoResponse> {
    match repo.list().await {
        Ok(tasks) => Ok(Json(tasks.into_iter().map(TaskResponse::from).collect())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Create a new task
#[utoipa::path(
    post,
    path = "/api/tasks",
    request_body = CreateTaskRequest,
    responses(
        (status = 201, description = "Task created successfully", body = TaskResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "tasks"
)]
pub async fn create_task<R: TaskRepository>(
    State(repo): State<Arc<R>>,
    Json(payload): Json<CreateTaskRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match repo.create(&payload.title, &payload.description).await {
        Ok(task) => Ok((StatusCode::CREATED, Json(TaskResponse::from(task)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Get a task by ID
#[utoipa::path(
    get,
    path = "/api/tasks/{id}",
    params(
        ("id" = i64, Path, description = "Task ID")
    ),
    responses(
        (status = 200, description = "Task found", body = TaskResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
    ),
    tag = "tasks"
)]
pub async fn get_task<R: TaskRepository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<i64>,
) -> Result<Json<TaskResponse>, impl IntoResponse> {
    match repo.get(id).await {
        Ok(task) => Ok(Json(TaskResponse::from(task))),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Task with id {} not found", id),
            }),
        )),
    }
}

/// Update a task
#[utoipa::path(
    put,
    path = "/api/tasks/{id}",
    params(
        ("id" = i64, Path, description = "Task ID")
    ),
    request_body = UpdateTaskRequest,
    responses(
        (status = 200, description = "Task updated successfully", body = TaskResponse),
        (status = 404, description = "Task not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "tasks"
)]
pub async fn update_task<R: TaskRepository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateTaskRequest>,
) -> Result<Json<TaskResponse>, impl IntoResponse> {
    match repo
        .update(
            id,
            payload.title.as_deref(),
            payload.description.as_deref(),
            payload.completed,
        )
        .await
    {
        Ok(task) => Ok(Json(TaskResponse::from(task))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("no rows") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: format!("Task with id {} not found", id),
                    }),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse { error: error_msg }),
                ))
            }
        }
    }
}

/// Delete a task
#[utoipa::path(
    delete,
    path = "/api/tasks/{id}",
    params(
        ("id" = i64, Path, description = "Task ID")
    ),
    responses(
        (status = 204, description = "Task deleted successfully"),
        (status = 404, description = "Task not found", body = ErrorResponse),
    ),
    tag = "tasks"
)]
pub async fn delete_task<R: TaskRepository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<i64>,
) -> Result<StatusCode, impl IntoResponse> {
    match repo.delete(id).await {
        Ok(deleted) => {
            if deleted {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: format!("Task with id {} not found", id),
                    }),
                ))
            }
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}
