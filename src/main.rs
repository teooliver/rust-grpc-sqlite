use rust_grpc_sqlite::{
    db, grpc_server,
    repository::{SqliteTaskRepository, SqliteUserRepository},
    rest_server,
};

use anyhow::Result;
use std::sync::Arc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing database...");
    let pool = db::init_db().await?;
    println!("Database initialized successfully");

    // Create repositories and wrap in Arc for sharing between servers
    let task_repository = Arc::new(SqliteTaskRepository::new(pool.clone()));
    let user_repository = Arc::new(SqliteUserRepository::new(pool));

    let grpc_task_repository = task_repository.clone();
    let grpc_user_repository = user_repository.clone();
    let rest_task_repository = task_repository.clone();
    let rest_user_repository = user_repository.clone();

    // Spawn gRPC server on port 50051 (binding to all interfaces)
    let grpc_addr = "[::]:50051".parse()?;
    let grpc_handle = tokio::spawn(async move {
        println!("Starting gRPC server on {}", grpc_addr);
        let task_service = grpc_server::TaskServiceImpl::new(grpc_task_repository).into_service();
        let user_service = grpc_server::UserServiceImpl::new(grpc_user_repository).into_service();

        // Set up reflection service
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(grpc_server::task::FILE_DESCRIPTOR_SET)
            .register_encoded_file_descriptor_set(grpc_server::user::FILE_DESCRIPTOR_SET)
            .build_v1()
            .expect("Failed to build reflection service");

        Server::builder()
            .add_service(task_service)
            .add_service(user_service)
            .add_service(reflection_service)
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Spawn REST server on port 3000 (binding to all interfaces)
    let rest_addr = "[::]:3000";
    let rest_handle = tokio::spawn(async move {
        println!("Starting REST server on {}", rest_addr);
        let app = rest_server::create_router(rest_task_repository, rest_user_repository);

        let listener = tokio::net::TcpListener::bind(rest_addr)
            .await
            .expect("Failed to bind REST server");

        axum::serve(listener, app)
            .await
            .expect("REST server failed");
    });

    println!("\nServers are running:");
    println!("  - gRPC server: [::]:50051 (accessible via localhost:50051 or 127.0.0.1:50051)");
    println!("  - REST server: [::]:3000 (accessible via http://localhost:3000)");
    println!("\nPress Ctrl+C to stop");

    // Wait for both servers
    tokio::try_join!(grpc_handle, rest_handle)?;

    Ok(())
}
