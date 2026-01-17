# gRPC + SQLite Task App

A simple task list application demonstrating:
- **gRPC** with [tonic](https://github.com/hyperium/tonic)
- **SQLite** with [sqlx](https://github.com/launchbadge/sqlx)

## Project Structure

```
.
├── proto/
│   ├── task.proto         # Task service Protocol Buffer definitions
│   └── user.proto         # User service Protocol Buffer definitions
├── src/
│   ├── main.rs            # Entry point, runs gRPC server
│   ├── db.rs              # SQLite database models and initialization
│   ├── grpc_server.rs     # gRPC service implementations
│   ├── controller/        # Database CRUD operations
│   ├── repository/        # Repository traits and implementations
│   └── service/           # Service layer implementations
├── build.rs               # Builds protobuf files
└── Cargo.toml
```

## Running the Application

```bash
cargo run
```

This starts the gRPC server on `[::]:50051` (accessible via `localhost:50051`).

The SQLite database file `tasks.db` will be created in the project root.

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
grpcurl -plaintext localhost:50051 list
```

### Task Operations

#### Create a task
```bash
grpcurl -plaintext -d '{"title": "Learn gRPC", "description": "Master tonic"}' \
  localhost:50051 task.TaskService/CreateTask
```

#### List all tasks
```bash
grpcurl -plaintext -d '{}' \
  localhost:50051 task.TaskService/ListTasks
```

#### Get a task
```bash
grpcurl -plaintext -d '{"id": 1}' \
  localhost:50051 task.TaskService/GetTask
```

#### Update a task
```bash
grpcurl -plaintext -d '{"id": 1, "completed": true}' \
  localhost:50051 task.TaskService/UpdateTask
```

#### Delete a task
```bash
grpcurl -plaintext -d '{"id": 1}' \
  localhost:50051 task.TaskService/DeleteTask
```

### User Operations

#### Create a user
```bash
grpcurl -plaintext -d '{"name": "John Doe", "email": "john@example.com"}' \
  localhost:50051 user.UserService/CreateUser
```

#### List all users
```bash
grpcurl -plaintext -d '{}' \
  localhost:50051 user.UserService/ListUsers
```

#### Get a user
```bash
grpcurl -plaintext -d '{"id": 1}' \
  localhost:50051 user.UserService/GetUser
```

#### Update a user
```bash
grpcurl -plaintext -d '{"id": 1, "name": "Jane Doe"}' \
  localhost:50051 user.UserService/UpdateUser
```

#### Delete a user
```bash
grpcurl -plaintext -d '{"id": 1}' \
  localhost:50051 user.UserService/DeleteUser
```

## Key Features

### SQLite with sqlx
- Async database operations
- Compile-time checked queries
- Connection pooling
- Auto-creates database schema on startup

### gRPC with tonic
- Protocol buffer definitions in `proto/`
- Full CRUD operations for Tasks and Users
- Type-safe client/server code generation
- gRPC reflection enabled for introspection

### Architecture
- **Repository pattern** for data access abstraction
- **Controller layer** for database operations
- **Service layer** for business logic
- Layered design with clear separation of concerns

## Running Tests

```bash
cargo test
```

This runs unit tests and gRPC integration tests.

## Dependencies

- `tonic` & `prost`: gRPC implementation
- `sqlx`: Async SQL toolkit
- `tokio`: Async runtime
- `serde` & `serde_json`: JSON serialization
- `anyhow`: Error handling
- `tonic-reflection`: gRPC reflection support
