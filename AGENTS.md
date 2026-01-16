# Architecture & Components

This document describes the architecture, components, and data flow of the rust-grpc-sqlite application.

## Overview

The application implements a dual-interface system with both gRPC and REST APIs sharing a common SQLite database. The architecture follows a layered approach with clear separation of concerns, supporting multiple entities (Tasks and Users).

## Project Structure

```
src/
├── controller/
│   ├── mod.rs          # Exports task and user modules
│   ├── task.rs         # Task CRUD functions
│   └── user.rs         # User CRUD functions
├── repository/
│   ├── mod.rs          # Exports repository traits and implementations
│   ├── task.rs         # TaskRepository trait and SqliteTaskRepository
│   └── user.rs         # UserRepository trait and SqliteUserRepository
├── db.rs               # Database models and initialization
├── grpc_server.rs      # gRPC service implementations
├── rest_server.rs      # REST API handlers and routes
├── lib.rs              # Module exports
└── main.rs             # Application entry point

proto/
├── task.proto          # Task service Protocol Buffer definition
└── user.proto          # User service Protocol Buffer definition

tests/
├── common/mod.rs       # Shared test utilities
├── grpc_integration.rs # gRPC integration tests
└── rest_integration.rs # REST integration tests
```

## System Architecture

```
┌─────────────┐     ┌─────────────┐
│   gRPC      │     │    REST     │
│   Client    │     │   Client    │
└──────┬──────┘     └──────┬──────┘
       │                   │
       │ :50051            │ :3000
       │                   │
┌──────▼───────────────────▼──────┐
│         Server Layer            │
│  ┌────────────┐  ┌────────────┐ │
│  │   gRPC     │  │    REST    │ │
│  │   Server   │  │   Server   │ │
│  └─────┬──────┘  └─────┬──────┘ │
│        │               │        │
│        └───────┬───────┘        │
│                │                │
│       ┌────────▼────────┐       │
│       │ Repository Layer│       │
│       │ (Task & User)   │       │
│       └────────┬────────┘       │
│                │                │
│       ┌────────▼────────┐       │
│       │ Controller Layer│       │
│       │ (Task & User)   │       │
│       └────────┬────────┘       │
│                │                │
│       ┌────────▼────────┐       │
│       │  Database Layer │       │
│       │    (db.rs)      │       │
│       └────────┬────────┘       │
└────────────────┼────────────────┘
                 │
         ┌───────▼────────┐
         │ SQLite Database│
         │   tasks.db     │
         └────────────────┘
```

## Core Components

### 1. Database Layer (`src/db.rs`)

**Responsibility**: Database initialization and model definitions

**Contents**:
- `init_db()` - Initializes SQLite connection pool and creates schema for all tables
- `TaskModel` - Database model for tasks
- `UserModel` - Database model for users

**Data Models**:
```rust
TaskModel {
    id: i64,
    title: String,
    description: String,
    completed: bool,
}

UserModel {
    id: i64,
    name: String,
    email: String,  // UNIQUE constraint
}
```

**Key Design Decisions**:
- Uses SQLx for compile-time checked SQL queries
- Connection pooling (max 5 connections) for concurrent access
- Automatic schema creation on startup
- Models are separate from CRUD operations

### 2. Controller Layer (`src/controller/`)

**Responsibility**: Direct database CRUD operations

**Structure**:
- `controller/task.rs` - Task CRUD functions
- `controller/user.rs` - User CRUD functions

**Task Functions**:
- `create_task(pool, title, description)` - Inserts new task
- `get_task(pool, id)` - Retrieves single task by ID
- `list_tasks(pool)` - Fetches all tasks ordered by ID (descending)
- `update_task(pool, id, title?, description?, completed?)` - Partial updates
- `delete_task(pool, id)` - Removes task from database

**User Functions**:
- `create_user(pool, name, email)` - Inserts new user
- `get_user(pool, id)` - Retrieves single user by ID
- `list_users(pool)` - Fetches all users ordered by ID (descending)
- `update_user(pool, id, name?, email?)` - Partial updates
- `delete_user(pool, id)` - Removes user from database

**Key Features**:
- RETURNING clause for atomic insert/update operations
- Partial updates via optional parameters
- Each entity has its own file for separation of concerns

### 3. Repository Layer (`src/repository/`)

**Responsibility**: Abstract data access with trait-based design

**Structure**:
- `repository/task.rs` - TaskRepository trait and SqliteTaskRepository
- `repository/user.rs` - UserRepository trait and SqliteUserRepository

**TaskRepository Trait**:
```rust
#[async_trait]
pub trait TaskRepository: Send + Sync {
    async fn create(&self, title: &str, description: &str) -> Result<TaskModel>;
    async fn get(&self, id: i64) -> Result<TaskModel>;
    async fn list(&self) -> Result<Vec<TaskModel>>;
    async fn update(&self, id: i64, title: Option<&str>, 
                    description: Option<&str>, completed: Option<bool>) -> Result<TaskModel>;
    async fn delete(&self, id: i64) -> Result<bool>;
}
```

**UserRepository Trait**:
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, name: &str, email: &str) -> Result<UserModel>;
    async fn get(&self, id: i64) -> Result<UserModel>;
    async fn list(&self) -> Result<Vec<UserModel>>;
    async fn update(&self, id: i64, name: Option<&str>, email: Option<&str>) -> Result<UserModel>;
    async fn delete(&self, id: i64) -> Result<bool>;
}
```

**Key Design Decisions**:
- Trait-based design enables dependency injection
- `Send + Sync` bounds for thread-safe sharing via `Arc`
- SQLite implementations provided (`SqliteTaskRepository`, `SqliteUserRepository`)
- Facilitates testing with mock repositories

### 4. gRPC Server (`src/grpc_server.rs`)

**Responsibility**: Protocol Buffer-based API implementation

**Services**:
- `TaskService` (defined in `proto/task.proto`)
- `UserService` (defined in `proto/user.proto`)

**Task Operations**:
- `CreateTask` - Creates new task from title and description
- `GetTask` - Retrieves task by ID
- `ListTasks` - Returns all tasks
- `UpdateTask` - Partial or full task updates
- `DeleteTask` - Removes task and returns success status

**User Operations**:
- `CreateUser` - Creates new user from name and email
- `GetUser` - Retrieves user by ID
- `ListUsers` - Returns all users
- `UpdateUser` - Partial or full user updates
- `DeleteUser` - Removes user and returns success status

**Key Features**:
- Tonic-based gRPC implementation
- gRPC reflection enabled for introspection
- Uses repository pattern for data access
- Error mapping to gRPC Status codes
- Model-to-Proto conversion helpers

**Error Handling**:
- `Status::internal` - Database failures
- `Status::not_found` - Missing resources

### 5. REST Server (`src/rest_server.rs`)

**Responsibility**: HTTP/JSON API implementation

**Task Routes**:
```
POST   /tasks       - Create new task
GET    /tasks       - List all tasks
GET    /tasks/:id   - Get specific task
PUT    /tasks/:id   - Update task
DELETE /tasks/:id   - Delete task
```

**User Routes**:
```
POST   /users       - Create new user
GET    /users       - List all users
GET    /users/:id   - Get specific user
PUT    /users/:id   - Update user
DELETE /users/:id   - Delete user
```

**Key Features**:
- Axum-based web framework
- JSON request/response serialization via Serde
- State management via `AppState` with both repositories
- Path parameter extraction for resource IDs
- Proper HTTP status codes (200, 204, 404, 500)

**Request/Response Models**:
- Separate DTOs for requests and responses
- Optional fields for partial updates

**Error Handling**:
- Custom `AppError` enum for clean error responses
- Automatic conversion from sqlx::Error and anyhow::Error
- HTTP 404 for not found, 500 for database errors

### 6. Application Entry Point (`src/main.rs`)

**Responsibility**: Server orchestration and initialization

**Key Operations**:
1. Initialize SQLite database pool
2. Create repositories for tasks and users
3. Clone repositories for both servers
4. Spawn gRPC server on `[::]:50051` with both services
5. Spawn REST server on `[::]:3000` with all routes
6. Run both servers concurrently via tokio

**Key Features**:
- Async runtime via Tokio
- Concurrent server execution with `tokio::spawn`
- Graceful startup logging
- IPv6 binding with IPv4 compatibility
- gRPC reflection for both services

## Data Flow

### Creating a Task (REST Example)

```
1. HTTP POST /tasks with JSON body
   ↓
2. rest_server::create_task handler
   ↓
3. task_repository.create(title, description)
   ↓
4. SQLite INSERT with RETURNING
   ↓
5. TaskModel returned
   ↓
6. Convert to TaskResponse
   ↓
7. JSON response to client
```

### Creating a User (gRPC Example)

```
1. gRPC CreateUser request
   ↓
2. UserServiceImpl::create_user handler
   ↓
3. user_repository.create(name, email)
   ↓
4. SQLite INSERT with RETURNING
   ↓
5. UserModel returned
   ↓
6. Convert to Proto User
   ↓
7. Proto response to client
```

## Technology Stack

### Core Frameworks
- **tonic** (0.12) - gRPC server implementation
- **prost** (0.13) - Protocol Buffer serialization
- **axum** (0.7) - Web framework for REST API
- **sqlx** (0.8) - Async SQL toolkit with compile-time checking
- **tokio** (1.x) - Async runtime

### Supporting Libraries
- **serde** + **serde_json** - JSON serialization
- **anyhow** - Ergonomic error handling
- **async-trait** - Async trait support
- **tonic-reflection** - gRPC service reflection

## Testing Strategy

### Unit Tests
Each layer includes comprehensive unit tests:
- **Controller tests** (`controller/task.rs`, `controller/user.rs`) - CRUD operations with in-memory SQLite
- **Repository tests** (`repository/task.rs`, `repository/user.rs`) - Repository trait implementations

### Integration Tests
- **gRPC tests** (`tests/grpc_integration.rs`) - Full gRPC service testing with real server
- **REST tests** (`tests/rest_integration.rs`) - HTTP endpoint testing via tower ServiceExt

### Test Utilities
Common test setup in `tests/common/mod.rs`:
- `setup_test_pool()` - In-memory SQLite pool
- `setup_test_repository()` / `setup_test_user_repository()` - Empty repository
- `setup_test_repository_with_data()` / `setup_test_user_repository_with_data()` - Pre-populated data

Test databases use in-memory SQLite (`:memory:`) for isolation and speed.

## Adding a New Entity

To add a new entity (e.g., `Project`):

1. **Proto Definition**: Create `proto/project.proto` with service and messages
2. **Build Configuration**: Update `build.rs` to compile the new proto
3. **Database Model**: Add `ProjectModel` to `src/db.rs` and update `init_db()`
4. **Controller**: Create `src/controller/project.rs` with CRUD functions
5. **Repository**: Create `src/repository/project.rs` with trait and implementation
6. **Update Exports**: Add to `controller/mod.rs` and `repository/mod.rs`
7. **gRPC Server**: Add `ProjectServiceImpl` to `src/grpc_server.rs`
8. **REST Server**: Add handlers and routes to `src/rest_server.rs`
9. **Main**: Register new services in `src/main.rs`
10. **Tests**: Add integration tests and update `tests/common/mod.rs`

## Security Considerations

Current implementation is suitable for development/learning:
- No authentication or authorization
- No input validation beyond type safety
- No rate limiting
- Binds to all interfaces (`[::]`)

For production use, consider:
- Authentication (JWT, API keys, mTLS)
- Input validation and sanitization
- Rate limiting
- TLS/SSL encryption
- Restricted network binding
- SQL injection protection (already handled via sqlx prepared statements)

## Extension Points

The architecture supports easy extension:

1. **Additional Entities**: Follow the "Adding a New Entity" guide above
2. **Business Logic**: Add validation in handlers before repository calls
3. **Caching**: Insert cache layer between handlers and repositories
4. **WebSocket Support**: Add axum websocket routes for real-time updates
5. **Authentication**: Add middleware to both server implementations
6. **Database Migration**: Integrate sqlx-cli for versioned migrations
7. **Alternative Databases**: Implement new repository types (e.g., PostgresTaskRepository)
