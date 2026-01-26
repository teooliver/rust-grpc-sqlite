use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use crate::db::UserModel;
use crate::repository::UserRepository;

use super::{CreateUserRequest, ErrorResponse, UpdateUserRequest, UserResponse};

pub fn user_routes<R: UserRepository + 'static>(repo: Arc<R>) -> Router {
    Router::new()
        .route("/users", get(list_users::<R>).post(create_user::<R>))
        .route(
            "/users/{id}",
            get(get_user::<R>)
                .put(update_user::<R>)
                .delete(delete_user::<R>),
        )
        .with_state(repo)
}

impl From<UserModel> for UserResponse {
    fn from(model: UserModel) -> Self {
        UserResponse {
            id: model.id,
            name: model.name,
            email: model.email,
        }
    }
}

/// List all users
#[utoipa::path(
    get,
    path = "/api/users",
    responses(
        (status = 200, description = "List of all users", body = Vec<UserResponse>),
    ),
    tag = "users"
)]
pub async fn list_users<R: UserRepository>(
    State(repo): State<Arc<R>>,
) -> Result<Json<Vec<UserResponse>>, impl IntoResponse> {
    match repo.list().await {
        Ok(users) => Ok(Json(users.into_iter().map(UserResponse::from).collect())),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "users"
)]
pub async fn create_user<R: UserRepository>(
    State(repo): State<Arc<R>>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match repo.create(&payload.name, &payload.email).await {
        Ok(user) => Ok((StatusCode::CREATED, Json(UserResponse::from(user)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )),
    }
}

/// Get a user by ID
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub async fn get_user<R: UserRepository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<i64>,
) -> Result<Json<UserResponse>, impl IntoResponse> {
    match repo.get(id).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(_) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("User with id {} not found", id),
            }),
        )),
    }
}

/// Update a user
#[utoipa::path(
    put,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "users"
)]
pub async fn update_user<R: UserRepository>(
    State(repo): State<Arc<R>>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, impl IntoResponse> {
    match repo
        .update(id, payload.name.as_deref(), payload.email.as_deref())
        .await
    {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(e) => {
            let error_msg = e.to_string();
            if error_msg.contains("no rows") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: format!("User with id {} not found", id),
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

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    params(
        ("id" = i64, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found", body = ErrorResponse),
    ),
    tag = "users"
)]
pub async fn delete_user<R: UserRepository>(
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
                        error: format!("User with id {} not found", id),
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
