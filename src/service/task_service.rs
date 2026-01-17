use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::db;
use crate::grpc_server::task::{
    task_service_server::{TaskService, TaskServiceServer},
    CreateTaskRequest, DeleteTaskRequest, DeleteTaskResponse, GetTaskRequest, ListTasksRequest,
    ListTasksResponse, Task, UpdateTaskRequest,
};
use crate::repository::TaskRepository;

pub struct TaskServiceImpl {
    repository: Arc<dyn TaskRepository>,
}

impl TaskServiceImpl {
    pub fn new(repository: Arc<dyn TaskRepository>) -> Self {
        Self { repository }
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

        let task = self
            .repository
            .create(&req.title, &req.description)
            .await
            .map_err(|e| Status::internal(format!("Failed to create task: {}", e)))?;

        Ok(Response::new(model_to_proto(task)))
    }

    async fn get_task(&self, request: Request<GetTaskRequest>) -> Result<Response<Task>, Status> {
        let req = request.into_inner();

        let task = self
            .repository
            .get(req.id)
            .await
            .map_err(|e| Status::not_found(format!("Task not found: {}", e)))?;

        Ok(Response::new(model_to_proto(task)))
    }

    async fn list_tasks(
        &self,
        _request: Request<ListTasksRequest>,
    ) -> Result<Response<ListTasksResponse>, Status> {
        let tasks = self
            .repository
            .list()
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

        let task = self
            .repository
            .update(
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

        let success = self
            .repository
            .delete(req.id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete task: {}", e)))?;

        Ok(Response::new(DeleteTaskResponse { success }))
    }
}
