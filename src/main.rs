mod db;
mod grpc_server;
mod rest_server;

use anyhow::Result;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Initializing database...");
    let pool = db::init_db().await?;
    println!("Database initialized successfully");

    // Clone pool for both servers
    let grpc_pool = pool.clone();
    let rest_pool = pool.clone();

    // Spawn gRPC server on port 50051 (binding to all interfaces)
    let grpc_addr = "[::]:50051".parse()?;
    let grpc_handle = tokio::spawn(async move {
        println!("Starting gRPC server on {}", grpc_addr);
        let task_service = grpc_server::TaskServiceImpl::new(grpc_pool).into_service();

        // Set up reflection service
        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(grpc_server::task::FILE_DESCRIPTOR_SET)
            .build_v1()
            .expect("Failed to build reflection service");

        Server::builder()
            .add_service(task_service)
            .add_service(reflection_service)
            .serve(grpc_addr)
            .await
            .expect("gRPC server failed");
    });

    // Spawn REST server on port 3000 (binding to all interfaces)
    let rest_addr = "[::]:3000";
    let rest_handle = tokio::spawn(async move {
        println!("Starting REST server on {}", rest_addr);
        let app = rest_server::create_router(rest_pool);

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
