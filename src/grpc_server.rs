use sqlx::SqlitePool;
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod todo {
    tonic::include_proto!("todo");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("todo_descriptor");
}

use todo::{
    todo_service_server::{TodoService, TodoServiceServer},
    CreateTodoRequest, DeleteTodoRequest, DeleteTodoResponse, GetTodoRequest, ListTodosRequest,
    ListTodosResponse, Todo, UpdateTodoRequest,
};

use crate::db;

pub struct TodoServiceImpl {
    pool: SqlitePool,
}

impl TodoServiceImpl {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn into_service(self) -> TodoServiceServer<Self> {
        TodoServiceServer::new(self)
    }
}

fn model_to_proto(model: db::TodoModel) -> Todo {
    Todo {
        id: model.id,
        title: model.title,
        description: model.description,
        completed: model.completed,
    }
}

#[tonic::async_trait]
impl TodoService for TodoServiceImpl {
    async fn create_todo(
        &self,
        request: Request<CreateTodoRequest>,
    ) -> Result<Response<Todo>, Status> {
        let req = request.into_inner();

        let todo = db::create_todo(&self.pool, &req.title, &req.description)
            .await
            .map_err(|e| Status::internal(format!("Failed to create todo: {}", e)))?;

        Ok(Response::new(model_to_proto(todo)))
    }

    async fn get_todo(&self, request: Request<GetTodoRequest>) -> Result<Response<Todo>, Status> {
        let req = request.into_inner();

        let todo = db::get_todo(&self.pool, req.id)
            .await
            .map_err(|e| Status::not_found(format!("Todo not found: {}", e)))?;

        Ok(Response::new(model_to_proto(todo)))
    }

    async fn list_todos(
        &self,
        _request: Request<ListTodosRequest>,
    ) -> Result<Response<ListTodosResponse>, Status> {
        let todos = db::list_todos(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Failed to list todos: {}", e)))?;

        let todos = todos.into_iter().map(model_to_proto).collect();

        Ok(Response::new(ListTodosResponse { todos }))
    }

    async fn update_todo(
        &self,
        request: Request<UpdateTodoRequest>,
    ) -> Result<Response<Todo>, Status> {
        let req = request.into_inner();

        let todo = db::update_todo(
            &self.pool,
            req.id,
            req.title.as_deref(),
            req.description.as_deref(),
            req.completed,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to update todo: {}", e)))?;

        Ok(Response::new(model_to_proto(todo)))
    }

    async fn delete_todo(
        &self,
        request: Request<DeleteTodoRequest>,
    ) -> Result<Response<DeleteTodoResponse>, Status> {
        let req = request.into_inner();

        let success = db::delete_todo(&self.pool, req.id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete todo: {}", e)))?;

        Ok(Response::new(DeleteTodoResponse { success }))
    }
}
