# Architecture & Components

This document describes the architecture, components, and data flow of the rust-grpc-sqlite task management application.

## Overview

The application implements a dual-interface task management system with both gRPC and REST APIs sharing a common SQLite database. The architecture follows a layered approach with clear separation of concerns.

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
│      Application Layer           │
│  ┌────────────┐  ┌─────────────┐│
│  │   gRPC     │  │    REST     ││
│  │   Server   │  │    Server   ││
│  └─────┬──────┘  └──────┬──────┘│
│        │                │       │
│        └────────┬────────┘       │
│                 │                │
│        ┌────────▼────────┐       │
│        │  Database Layer │       │
│        │    (db.rs)      │       │
│        └────────┬────────┘       │
└─────────────────┼────────────────┘
                  │
          ┌───────▼────────┐
          │ SQLite Database│
          │   tasks.db     │
          └────────────────┘
```

## Core Components

### 1. Database Layer (`src/db.rs`)

**Responsibility**: Data persistence and SQLite operations

**Key Functions**:
- `init_db()` - Initializes SQLite connection pool and creates schema
- `create_task()` - Inserts new task into database
- `get_task()` - Retrieves single task by ID
- `list_tasks()` - Fetches all tasks ordered by ID (descending)
- `update_task()` - Updates task fields (partial updates supported)
- `delete_task()` - Removes task from database

**Data Model**:
```rust
TaskModel {
    id: i64,
    title: String,
    description: String,
    completed: bool,
}
```

**Key Design Decisions**:
- Uses SQLx for compile-time checked SQL queries
- Connection pooling (max 5 connections) for concurrent access
- Automatic schema creation on startup
- RETURNING clause for atomic insert/update operations
- Partial updates via optional parameters

### 2. gRPC Server (`src/grpc_server.rs`)

**Responsibility**: Protocol Buffer-based API implementation

**Service Definition**: `TaskService` (defined in `proto/task.proto`)

**Operations**:
- `CreateTask` - Creates new task from title and description
- `GetTask` - Retrieves task by ID
- `ListTasks` - Returns all tasks
- `UpdateTask` - Partial or full task updates
- `DeleteTask` - Removes task and returns success status

**Key Features**:
- Tonic-based gRPC implementation
- gRPC reflection enabled for introspection
- Shares SqlitePool with REST server
- Error mapping to gRPC Status codes
- Model-to-Proto conversion layer

**Error Handling**:
- `Status::internal` - Database failures
- `Status::not_found` - Missing tasks

### 3. REST Server (`src/rest_server.rs`)

**Responsibility**: HTTP/JSON API implementation

**Routes**:
```
POST   /tasks       - Create new task
GET    /tasks       - List all tasks
GET    /tasks/:id   - Get specific task
PUT    /tasks/:id   - Update task
DELETE /tasks/:id   - Delete task
```

**Key Features**:
- Axum-based web framework
- JSON request/response serialization via Serde
- State management via `AppState` wrapper
- Path parameter extraction for resource IDs
- Proper HTTP status codes (200, 204, 404, 500)

**Request/Response Models**:
- `CreateTaskRequest` - title, description
- `UpdateTaskRequest` - optional title, description, completed
- `TaskResponse` - id, title, description, completed

**Error Handling**:
- Custom `AppError` enum for clean error responses
- Automatic conversion from sqlx::Error and anyhow::Error
- HTTP 404 for not found, 500 for database errors

### 4. Application Entry Point (`src/main.rs`)

**Responsibility**: Server orchestration and initialization

**Key Operations**:
1. Initialize SQLite database pool
2. Clone pool for both servers
3. Spawn gRPC server on `[::]:50051`
4. Spawn REST server on `[::]:3000`
5. Run both servers concurrently via tokio

**Key Features**:
- Async runtime via Tokio
- Concurrent server execution with `tokio::spawn`
- Graceful startup logging
- IPv6 binding with IPv4 compatibility

## Data Flow

### Creating a Task (REST Example)

```
1. HTTP POST /tasks
   ↓
2. rest_server::create_task handler
   ↓
3. db::create_task(&pool, title, description)
   ↓
4. SQLite INSERT with RETURNING
   ↓
5. TaskModel returned
   ↓
6. Convert to TaskResponse
   ↓
7. JSON response to client
```

### Creating a Task (gRPC Example)

```
1. gRPC CreateTask request
   ↓
2. grpc_server::create_task handler
   ↓
3. db::create_task(&pool, title, description)
   ↓
4. SQLite INSERT with RETURNING
   ↓
5. TaskModel returned
   ↓
6. Convert to Proto Task
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
- **tonic-reflection** - gRPC service reflection

## Shared State

Both servers share:
- **SQLite connection pool** - Cloned from main pool
- **Database file** - `tasks.db` in project root
- **Data consistency** - Tasks created via one interface are immediately visible in the other

## Testing Strategy

Database layer includes comprehensive unit tests:
- Task CRUD operations
- Partial updates
- Not found scenarios
- Task independence
- Default values

Test database uses in-memory SQLite (`:memory:`) for isolation.

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

1. **Additional Fields**: Modify proto definition, TaskModel, and migrations
2. **Business Logic**: Add validation in handlers before db calls
3. **Caching**: Insert cache layer between handlers and database
4. **WebSocket Support**: Add axum websocket routes for real-time updates
5. **Authentication**: Add middleware to both server implementations
6. **Database Migration**: Integrate sqlx-cli for versioned migrations
