use rust_grpc_sqlite::{
    db, grpc_server,
    repository::{SqliteTaskRepository, SqliteUserRepository},
    rest::{
        CreateTaskRequest, CreateUserRequest, ErrorResponse, TaskResponse, UpdateTaskRequest,
        UpdateUserRequest, UserResponse,
    },
    service::{TaskServiceImpl, UserServiceImpl},
};

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tower_http::cors::{Any, CorsLayer};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        rust_grpc_sqlite::rest::task_handlers::list_tasks,
        rust_grpc_sqlite::rest::task_handlers::create_task,
        rust_grpc_sqlite::rest::task_handlers::get_task,
        rust_grpc_sqlite::rest::task_handlers::update_task,
        rust_grpc_sqlite::rest::task_handlers::delete_task,
        rust_grpc_sqlite::rest::user_handlers::list_users,
        rust_grpc_sqlite::rest::user_handlers::create_user,
        rust_grpc_sqlite::rest::user_handlers::get_user,
        rust_grpc_sqlite::rest::user_handlers::update_user,
        rust_grpc_sqlite::rest::user_handlers::delete_user,
    ),
    components(
        schemas(
            TaskResponse,
            CreateTaskRequest,
            UpdateTaskRequest,
            UserResponse,
            CreateUserRequest,
            UpdateUserRequest,
            ErrorResponse,
        )
    ),
    tags(
        (name = "tasks", description = "Task management endpoints"),
        (name = "users", description = "User management endpoints")
    ),
    info(
        title = "Rust gRPC SQLite REST API",
        version = "1.0.0",
        description = "REST API layer for the Rust gRPC SQLite application"
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing database...");
    let pool = db::init_db().await?;
    println!("Database initialized successfully");

    // Create repositories and wrap in Arc for sharing
    let task_repository = Arc::new(SqliteTaskRepository::new(pool.clone()));
    let user_repository = Arc::new(SqliteUserRepository::new(pool));

    // Clone repositories for REST API
    let task_repo_rest = task_repository.clone();
    let user_repo_rest = user_repository.clone();

    // Spawn gRPC server
    let grpc_handle = tokio::spawn(async move {
        let grpc_addr = "[::]:50051".parse().unwrap();

        let task_service = TaskServiceImpl::new(task_repository).into_service();
        let user_service = UserServiceImpl::new(user_repository).into_service();

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(grpc_server::task::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(grpc_server::user::FILE_DESCRIPTOR_SET)
            .build_v1()
            .expect("Failed to build reflection service");

        println!("gRPC server listening on {}", grpc_addr);

        Server::builder()
            .accept_http1(true)
            .layer(GrpcWebLayer::new())
            .add_service(task_service)
            .add_service(user_service)
            .add_service(reflection_service)
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Build REST API router
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest(
            "/api",
            rust_grpc_sqlite::rest::task_routes(task_repo_rest)
                .merge(rust_grpc_sqlite::rest::user_routes(user_repo_rest)),
        )
        .layer(cors);

    // Start REST server
    let rest_addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(rest_addr).await?;

    println!("\n========================================");
    println!("Servers are running:");
    println!("========================================");
    println!("  gRPC:    [::]:50051");
    println!("  REST:    http://localhost:3000");
    println!("  Swagger: http://localhost:3000/swagger-ui/");
    println!("========================================");
    println!("\nPress Ctrl+C to stop");

    let rest_handle = tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("REST server failed");
    });

    // Wait for both servers
    tokio::select! {
        _ = grpc_handle => println!("gRPC server stopped"),
        _ = rest_handle => println!("REST server stopped"),
    }

    Ok(())
}
