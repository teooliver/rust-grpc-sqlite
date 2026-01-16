use std::sync::Arc;
use tonic::{Request, Response, Status};

// Include the generated proto code
pub mod task {
    tonic::include_proto!("task");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("task_descriptor");
}

pub mod user {
    tonic::include_proto!("user");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("user_descriptor");
}

use task::{
    task_service_server::{TaskService, TaskServiceServer},
    CreateTaskRequest, DeleteTaskRequest, DeleteTaskResponse, GetTaskRequest, ListTasksRequest,
    ListTasksResponse, Task, UpdateTaskRequest,
};

use user::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, DeleteUserRequest, DeleteUserResponse, GetUserRequest, ListUsersRequest,
    ListUsersResponse, UpdateUserRequest, User,
};

use crate::db;
use crate::repository::{TaskRepository, UserRepository};

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

// User Service Implementation

pub struct UserServiceImpl {
    repository: Arc<dyn UserRepository>,
}

impl UserServiceImpl {
    pub fn new(repository: Arc<dyn UserRepository>) -> Self {
        Self { repository }
    }

    pub fn into_service(self) -> UserServiceServer<Self> {
        UserServiceServer::new(self)
    }
}

fn user_model_to_proto(model: db::UserModel) -> User {
    User {
        id: model.id,
        name: model.name,
        email: model.email,
    }
}

#[tonic::async_trait]
impl UserService for UserServiceImpl {
    async fn create_user(
        &self,
        request: Request<CreateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let req = request.into_inner();

        let user = self
            .repository
            .create(&req.name, &req.email)
            .await
            .map_err(|e| Status::internal(format!("Failed to create user: {}", e)))?;

        Ok(Response::new(user_model_to_proto(user)))
    }

    async fn get_user(&self, request: Request<GetUserRequest>) -> Result<Response<User>, Status> {
        let req = request.into_inner();

        let user = self
            .repository
            .get(req.id)
            .await
            .map_err(|e| Status::not_found(format!("User not found: {}", e)))?;

        Ok(Response::new(user_model_to_proto(user)))
    }

    async fn list_users(
        &self,
        _request: Request<ListUsersRequest>,
    ) -> Result<Response<ListUsersResponse>, Status> {
        let users = self
            .repository
            .list()
            .await
            .map_err(|e| Status::internal(format!("Failed to list users: {}", e)))?;

        let users = users.into_iter().map(user_model_to_proto).collect();

        Ok(Response::new(ListUsersResponse { users }))
    }

    async fn update_user(
        &self,
        request: Request<UpdateUserRequest>,
    ) -> Result<Response<User>, Status> {
        let req = request.into_inner();

        let user = self
            .repository
            .update(req.id, req.name.as_deref(), req.email.as_deref())
            .await
            .map_err(|e| Status::internal(format!("Failed to update user: {}", e)))?;

        Ok(Response::new(user_model_to_proto(user)))
    }

    async fn delete_user(
        &self,
        request: Request<DeleteUserRequest>,
    ) -> Result<Response<DeleteUserResponse>, Status> {
        let req = request.into_inner();

        let success = self
            .repository
            .delete(req.id)
            .await
            .map_err(|e| Status::internal(format!("Failed to delete user: {}", e)))?;

        Ok(Response::new(DeleteUserResponse { success }))
    }
}
