# gRPC + SQLite + Axum Task App

A simple task list application demonstrating:
- **gRPC** with [tonic](https://github.com/hyperium/tonic)
- **SQLite** with [sqlx](https://github.com/launchbadge/sqlx)

- **REST API** with [axum](https://github.com/tokio-rs/axum)

## Project Structure

```
.
├── proto/
│   └── task.proto          # Protocol buffer definitions
├── src/
│   ├── main.rs            # Entry point, runs both servers
│   ├── db.rs              # SQLite database operations
│   ├── grpc_server.rs     # gRPC service implementation
│   └── rest_server.rs     # REST API implementation
├── build.rs               # Builds protobuf files
└── Cargo.toml
```

## Running the Application

```bash
cargo run
```

This starts two servers:
- **gRPC server**: `127.0.0.1:50051`
- **REST server**: `127.0.0.1:3000`

The SQLite database file `tasks.db` will be created in the project root.

## REST API Examples

### Create a task
```bash
curl -X POST http://127.0.0.1:3000/tasks \
  -H "Content-Type: application/json" \
  -d '{"title": "Learn Rust", "description": "Study tonic, sqlx, and axum"}'
```

### List all tasks
```bash
curl http://127.0.0.1:3000/tasks
```

### Get a specific task
```bash
curl http://127.0.0.1:3000/tasks/1
```

### Update a task
```bash
curl -X PUT http://127.0.0.1:3000/tasks/1 \
  -H "Content-Type: application/json" \
  -d '{"completed": true}'
```

### Delete a task
```bash
curl -X DELETE http://127.0.0.1:3000/tasks/1
```

## gRPC Examples

You can use [grpcurl](https://github.com/fullstorydev/grpcurl) or [grpcui](https://github.com/fullstorydev/grpcui) to test the gRPC API.

### Using grpcui (Web UI)

The easiest way to explore the gRPC API is with grpcui:

```bash
grpcui -plaintext localhost:50051
```

This will open a web browser with an interactive UI to call all the gRPC methods.

### Using grpcurl (Command Line)

For command-line testing, use grpcurl:

### List services
```bash
grpcurl -plaintext 127.0.0.1:50051 list
```

### Create a task
```bash
grpcurl -plaintext -d '{"title": "Learn gRPC", "description": "Master tonic"}' \
  127.0.0.1:50051 task.TaskService/CreateTask
```

### List all tasks
```bash
grpcurl -plaintext -d '{}' \
  127.0.0.1:50051 task.TaskService/ListTasks
```

### Get a task
```bash
grpcurl -plaintext -d '{"id": 1}' \
  127.0.0.1:50051 task.TaskService/GetTask
```

### Update a task
```bash
grpcurl -plaintext -d '{"id": 1, "completed": true}' \
  127.0.0.1:50051 task.TaskService/UpdateTask
```

### Delete a task
```bash
grpcurl -plaintext -d '{"id": 1}' \
  127.0.0.1:50051 task.TaskService/DeleteTask
```

## Key Features

### SQLite with sqlx
- Async database operations
- Compile-time checked queries
- Connection pooling
- Auto-creates database schema on startup

### gRPC with tonic
- Protocol buffer definitions in `proto/todo.proto`
- Full CRUD operations
- Type-safe client/server code generation
- Async streaming support (not used in this simple example)

### REST API with axum
- JSON request/response handling
- Path parameters for resource IDs
- Proper HTTP status codes
- Error handling with custom error types

## Learning Points

1. **Database Layer** (`src/db.rs`): Pure database operations with sqlx
2. **gRPC Service** (`src/grpc_server.rs`): Implements the protobuf service definition
3. **REST API** (`src/rest_server.rs`): HTTP handlers using axum extractors
4. **Main** (`src/main.rs`): Runs both servers concurrently using tokio

Both servers share the same SQLite database, so tasks created via REST are visible via gRPC and vice versa.

## Dependencies

- `tonic` & `prost`: gRPC implementation
- `sqlx`: Async SQL toolkit
- `axum`: Web framework
- `tokio`: Async runtime
- `serde` & `serde_json`: JSON serialization
- `anyhow`: Error handling
