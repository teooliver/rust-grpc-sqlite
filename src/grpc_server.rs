use sqlx::SqlitePool;
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod task {
    tonic::include_proto!("task");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("task_descriptor");
}

use task::{
    task_service_server::{TaskService, TaskServiceServer},
    CreateTaskRequest, DeleteTaskRequest, DeleteTaskResponse, GetTaskRequest, ListTasksRequest,
    ListTasksResponse, Task, UpdateTaskRequest,
};

use crate::db;

pub struct TaskServiceImpl {
    pool: SqlitePool,
}

impl TaskServiceImpl {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn into_service(self) -> TaskServiceServer<Self> {
        TaskServiceServer::new(self)
    }
}

fn model_to_proto(model: db::TaskModel) -> Task {
    Task {
        id: model.id,
        title: model.title,
        description: model.description,
        completed: model.completed,
    }
}

#[tonic::async_trait]
impl TaskService for TaskServiceImpl {
    async fn create_task(
        &self,
        request: Request<CreateTaskRequest>,
    ) -> Result<Response<Task>, Status> {
        let req = request.into_inner();

        let task = db::create_task(&self.pool, &req.title, &req.description)
            .await
            .map_err(|e| Status::internal(format!("Failed to create task: {}", e)))?;

        Ok(Response::new(model_to_proto(task)))
    }

    async fn get_task(&self, request: Request<GetTaskRequest>) -> Result<Response<Task>, Status> {
        let req = request.into_inner();

        let task = db::get_task(&self.pool, req.id)
            .await
            .map_err(|e| Status::not_found(format!("Task not found: {}", e)))?;

        Ok(Response::new(model_to_proto(task)))
    }

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let tasks = db::list_tasks(&self.pool)
            .await
            .map_err(|e| Status::internal(format!("Failed to list tasks: {}", e)))?;

        let tasks = tasks.into_iter().map(model_to_proto).collect();

        Ok(Response::new(ListTasksResponse { tasks }))
    }

    async fn update_task(
        &self,
        request: Request<UpdateTaskRequest>,
    ) -> Result<Response<Task>, Status> {
        let req = request.into_inner();

        let task = db::update_task(
            &self.pool,
            req.id,
            req.title.as_deref(),
            req.description.as_deref(),
            req.completed,
        )
        .await
        .map_err(|e| Status::internal(format!("Failed to update task: {}", e)))?;

        Ok(Response::new(model_to_proto(task)))
    }

    async fn delete_task(
        &self,
        request: Request<DeleteTaskRequest>,
    ) -> Result<Response<DeleteTaskResponse>, Status> {
        let req = request.into_inner();

        let success = db::delete_task(&self.pool, req.id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete task: {}", e)))?;

        Ok(Response::new(DeleteTaskResponse { success }))
    }
}
