use rust_grpc_sqlite::{
    db, grpc_server,
    repository::{SqliteTaskRepository, SqliteUserRepository},
    service::{TaskServiceImpl, UserServiceImpl},
};

use anyhow::Result;
use std::sync::Arc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing database...");
    let pool = db::init_db().await?;
    println!("Database initialized successfully");

    // Create repositories and wrap in Arc for sharing
    let task_repository = Arc::new(SqliteTaskRepository::new(pool.clone()));
    let user_repository = Arc::new(SqliteUserRepository::new(pool));

    // Start gRPC server on port 50051 (binding to all interfaces)
    let grpc_addr = "[::]:50051".parse()?;
    println!("Starting gRPC server on {}", grpc_addr);

    let task_service = TaskServiceImpl::new(task_repository).into_service();
    let user_service = UserServiceImpl::new(user_repository).into_service();

    // Set up reflection service
    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(grpc_server::task::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(grpc_server::user::FILE_DESCRIPTOR_SET)
        .build_v1()
        .expect("Failed to build reflection service");

    println!("\ngRPC server is running:");
    println!("  Address: [::]:50051 (accessible via localhost:50051 or 127.0.0.1:50051)");
    println!("\nPress Ctrl+C to stop");

    Server::builder()
        .add_service(task_service)
        .add_service(user_service)
        .add_service(reflection_service)
        .serve(grpc_addr)
        .await?;

    Ok(())
}
