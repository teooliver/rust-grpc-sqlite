# Decision: Adding a REST API Layer

## Context

This project is a Rust backend using gRPC with Tonic for service communication. While gRPC offers excellent performance and type safety through Protocol Buffers, we needed to evaluate the best approach for client-side communication, particularly for web browsers.

## Options Considered

### Option 1: gRPC-Web via `tonic-web`

The `tonic-web` crate enables browsers to communicate with gRPC services by translating the gRPC-Web protocol (HTTP/1.1) to standard gRPC (HTTP/2).

**Pros:**
- Single protocol for all clients
- Maintains protobuf type safety end-to-end
- No duplicate endpoint definitions

**Cons:**
- Requires gRPC-Web client libraries (e.g., `grpc-web`, `connect-web`)
- Additional complexity in frontend tooling (protobuf code generation)
- Limited to unary and server-streaming calls (no client or bidirectional streaming)
- Less familiar to frontend developers compared to REST

### Option 2: REST API Layer with Axum

Add a separate REST/JSON API layer using Axum that reuses the existing repository layer.

**Pros:**
- Simpler client integration (standard `fetch` or any HTTP client)
- No protobuf tooling required on the frontend
- Swagger/OpenAPI documentation for easy API exploration
- Familiar patterns for most developers
- Easy to test with tools like curl, Postman, or browser

**Cons:**
- Duplicate endpoint definitions (gRPC + REST)
- Additional code to maintain
- Loses protobuf type safety on the REST boundary

## Decision

**We chose to implement a REST API layer** for client applications while keeping the gRPC interface available.

### Rationale

1. **Simplicity**: REST with JSON is the path of least resistance for web clients. No additional build tooling, no protobuf compilation step, and standard browser APIs work out of the box.

2. **Developer Experience**: Swagger UI provides immediate, interactive API documentation. Developers can explore and test endpoints without writing any code.

3. **Flexibility**: Different clients can choose the protocol that best fits their needs:
   - Web browsers → REST API (port 3000)
   - Backend services → gRPC (port 50051)
   - Mobile apps → Either option

### Note on `tonic-web`

We've kept the `tonic-web` integration in place despite choosing REST for the primary client interface. This allows us to:

- Experiment with gRPC-Web features when needed
- Support clients that prefer the gRPC-Web protocol
- Evaluate the approach for future projects

The `tonic-web` layer adds minimal overhead and doesn't interfere with the REST API.

## Architecture

```
┌─────────────────┐     ┌─────────────────┐
│   Web Browser   │     │ Backend Service │
└────────┬────────┘     └────────┬────────┘
         │                       │
         │ HTTP/JSON             │ gRPC (HTTP/2)
         │                       │
         ▼                       ▼
┌─────────────────────────────────────────┐
│              Rust Backend               │
├─────────────────┬───────────────────────┤
│   REST (Axum)   │   gRPC (Tonic)        │
│   Port 3000     │   Port 50051          │
├─────────────────┴───────────────────────┤
│           Repository Layer              │
├─────────────────────────────────────────┤
│           SQLite Database               │
└─────────────────────────────────────────┘
```

## Endpoints

### REST API (Port 3000)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/tasks` | List all tasks |
| POST | `/api/tasks` | Create a task |
| GET | `/api/tasks/{id}` | Get task by ID |
| PUT | `/api/tasks/{id}` | Update a task |
| DELETE | `/api/tasks/{id}` | Delete a task |
| GET | `/api/users` | List all users |
| POST | `/api/users` | Create a user |
| GET | `/api/users/{id}` | Get user by ID |
| PUT | `/api/users/{id}` | Update a user |
| DELETE | `/api/users/{id}` | Delete a user |

**Swagger UI**: http://localhost:3000/swagger-ui/

### gRPC (Port 50051)

- `TaskService`: CreateTask, GetTask, ListTasks, UpdateTask, DeleteTask
- `UserService`: CreateUser, GetUser, ListUsers, UpdateUser, DeleteUser

## Future Considerations

- If frontend complexity grows and type safety becomes critical, we may revisit gRPC-Web with a code generation pipeline
- The REST layer could be auto-generated from protobufs using tools like `grpc-gateway` patterns if we want to reduce duplication
- Consider adding GraphQL as another option if query flexibility becomes important
