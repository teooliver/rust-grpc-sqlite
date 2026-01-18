use std::sync::Arc;
use tonic::{Request, Response, Status};

use crate::db;
use crate::grpc_server::user::{
    user_service_server::{UserService, UserServiceServer},
    CreateUserRequest, CreateUserResponse, DeleteUserRequest, DeleteUserResponse, GetUserRequest,
    GetUserResponse, ListUsersRequest, ListUsersResponse, UpdateUserRequest, UpdateUserResponse,
    User,
};
use crate::repository::UserRepository;

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
    ) -> Result<Response<CreateUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .repository
            .create(&req.name, &req.email)
            .await
            .map_err(|e| Status::internal(format!("Failed to create user: {}", e)))?;

        Ok(Response::new(CreateUserResponse {
            user: Some(user_model_to_proto(user)),
        }))
    }

    async fn get_user(
        &self,
        request: Request<GetUserRequest>,
    ) -> Result<Response<GetUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .repository
            .get(req.id)
            .await
            .map_err(|e| Status::not_found(format!("User not found: {}", e)))?;

        Ok(Response::new(GetUserResponse {
            user: Some(user_model_to_proto(user)),
        }))
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
    ) -> Result<Response<UpdateUserResponse>, Status> {
        let req = request.into_inner();

        let user = self
            .repository
            .update(req.id, req.name.as_deref(), req.email.as_deref())
            .await
            .map_err(|e| Status::internal(format!("Failed to update user: {}", e)))?;

        Ok(Response::new(UpdateUserResponse {
            user: Some(user_model_to_proto(user)),
        }))
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
