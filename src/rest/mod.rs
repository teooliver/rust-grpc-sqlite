pub mod task_handlers;
pub mod user_handlers;

pub use task_handlers::task_routes;
pub use user_handlers::user_routes;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// ============================================================================
// Task DTOs
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: i64,
    pub title: String,
    pub description: String,
    pub completed: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskRequest {
    pub title: String,
    pub description: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
}

// ============================================================================
// User DTOs
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub email: Option<String>,
}

// ============================================================================
// Error Response
// ============================================================================

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}
