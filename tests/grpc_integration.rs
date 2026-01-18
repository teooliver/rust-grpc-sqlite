mod common;

use rust_grpc_sqlite::grpc_server::task::{
    task_service_client::TaskServiceClient, CreateTaskRequest, DeleteTaskRequest, GetTaskRequest,
    ListTasksRequest, UpdateTaskRequest,
};
use rust_grpc_sqlite::grpc_server::user::{
    user_service_client::UserServiceClient, CreateUserRequest, DeleteUserRequest, GetUserRequest,
    ListUsersRequest, UpdateUserRequest,
};
use rust_grpc_sqlite::service::{TaskServiceImpl, UserServiceImpl};
use tonic::transport::{Channel, Server};

async fn setup_grpc_client() -> (TaskServiceClient<Channel>, tokio::task::JoinHandle<()>) {
    let repository = common::setup_test_repository().await;
    let service = TaskServiceImpl::new(repository).into_service();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .unwrap();

    (TaskServiceClient::new(channel), handle)
}

async fn setup_grpc_client_with_data() -> (TaskServiceClient<Channel>, tokio::task::JoinHandle<()>)
{
    let repository = common::setup_test_repository_with_data().await;
    let service = TaskServiceImpl::new(repository).into_service();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .unwrap();

    (TaskServiceClient::new(channel), handle)
}

#[tokio::test]
async fn test_create_task_grpc() {
    let (mut client, _handle) = setup_grpc_client().await;

    let request = tonic::Request::new(CreateTaskRequest {
        title: "Test Task".to_string(),
        description: "Test Description".to_string(),
    });

    let response = client.create_task(request).await.unwrap();
    let task = response.into_inner().task.unwrap();

    assert_eq!(task.title, "Test Task");
    assert_eq!(task.description, "Test Description");
    assert_eq!(task.completed, false);
    assert!(task.id > 0);
}

#[tokio::test]
async fn test_get_task_grpc() {
    let (mut client, _handle) = setup_grpc_client_with_data().await;

    let request = tonic::Request::new(GetTaskRequest { id: 1 });

    let response = client.get_task(request).await.unwrap();
    let task = response.into_inner().task.unwrap();

    assert_eq!(task.id, 1);
    assert_eq!(task.title, "Test Task 1");
    assert_eq!(task.description, "Description 1");
    assert_eq!(task.completed, false);
}

#[tokio::test]
async fn test_get_task_not_found_grpc() {
    let (mut client, _handle) = setup_grpc_client().await;

    let request = tonic::Request::new(GetTaskRequest { id: 999 });

    let result = client.get_task(request).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_list_tasks_grpc() {
    let (mut client, _handle) = setup_grpc_client_with_data().await;

    let request = tonic::Request::new(ListTasksRequest {});

    let response = client.list_tasks(request).await.unwrap();
    let tasks = response.into_inner().tasks;

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0].id, 2);
    assert_eq!(tasks[1].id, 1);
}

#[tokio::test]
async fn test_list_tasks_empty_grpc() {
    let (mut client, _handle) = setup_grpc_client().await;

    let request = tonic::Request::new(ListTasksRequest {});

    let response = client.list_tasks(request).await.unwrap();
    let tasks = response.into_inner().tasks;

    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn test_update_task_grpc() {
    let (mut client, _handle) = setup_grpc_client_with_data().await;

    let request = tonic::Request::new(UpdateTaskRequest {
        id: 1,
        title: Some("Updated Task".to_string()),
        description: Some("Updated Description".to_string()),
        completed: Some(true),
    });

    let response = client.update_task(request).await.unwrap();
    let task = response.into_inner().task.unwrap();

    assert_eq!(task.id, 1);
    assert_eq!(task.title, "Updated Task");
    assert_eq!(task.description, "Updated Description");
    assert_eq!(task.completed, true);
}

#[tokio::test]
async fn test_update_task_partial_grpc() {
    let (mut client, _handle) = setup_grpc_client_with_data().await;

    let request = tonic::Request::new(UpdateTaskRequest {
        id: 1,
        title: None,
        description: None,
        completed: Some(true),
    });

    let response = client.update_task(request).await.unwrap();
    let task = response.into_inner().task.unwrap();

    assert_eq!(task.id, 1);
    assert_eq!(task.title, "Test Task 1");
    assert_eq!(task.description, "Description 1");
    assert_eq!(task.completed, true);
}

#[tokio::test]
async fn test_update_task_not_found_grpc() {
    let (mut client, _handle) = setup_grpc_client().await;

    let request = tonic::Request::new(UpdateTaskRequest {
        id: 999,
        title: Some("Updated".to_string()),
        description: None,
        completed: None,
    });

    let result = client.update_task(request).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_task_grpc() {
    let (mut client, _handle) = setup_grpc_client_with_data().await;

    let request = tonic::Request::new(DeleteTaskRequest { id: 1 });

    let response = client.delete_task(request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.success, true);

    let get_request = tonic::Request::new(GetTaskRequest { id: 1 });
    let get_result = client.get_task(get_request).await;
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_delete_task_not_found_grpc() {
    let (mut client, _handle) = setup_grpc_client().await;

    let request = tonic::Request::new(DeleteTaskRequest { id: 999 });

    let response = client.delete_task(request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.success, false);
}

// User gRPC tests

async fn setup_user_grpc_client() -> (UserServiceClient<Channel>, tokio::task::JoinHandle<()>) {
    let repository = common::setup_test_user_repository().await;
    let service = UserServiceImpl::new(repository).into_service();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .unwrap();

    (UserServiceClient::new(channel), handle)
}

async fn setup_user_grpc_client_with_data(
) -> (UserServiceClient<Channel>, tokio::task::JoinHandle<()>) {
    let repository = common::setup_test_user_repository_with_data().await;
    let service = UserServiceImpl::new(repository).into_service();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(service)
            .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
            .await
            .unwrap();
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let channel = Channel::from_shared(format!("http://{}", addr))
        .unwrap()
        .connect()
        .await
        .unwrap();

    (UserServiceClient::new(channel), handle)
}

#[tokio::test]
async fn test_create_user_grpc() {
    let (mut client, _handle) = setup_user_grpc_client().await;

    let request = tonic::Request::new(CreateUserRequest {
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    });

    let response = client.create_user(request).await.unwrap();
    let user = response.into_inner().user.unwrap();

    assert_eq!(user.name, "John Doe");
    assert_eq!(user.email, "john@example.com");
    assert!(user.id > 0);
}

#[tokio::test]
async fn test_get_user_grpc() {
    let (mut client, _handle) = setup_user_grpc_client_with_data().await;

    let request = tonic::Request::new(GetUserRequest { id: 1 });

    let response = client.get_user(request).await.unwrap();
    let user = response.into_inner().user.unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "John Doe");
    assert_eq!(user.email, "john@example.com");
}

#[tokio::test]
async fn test_get_user_not_found_grpc() {
    let (mut client, _handle) = setup_user_grpc_client().await;

    let request = tonic::Request::new(GetUserRequest { id: 999 });

    let result = client.get_user(request).await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::NotFound);
}

#[tokio::test]
async fn test_list_users_grpc() {
    let (mut client, _handle) = setup_user_grpc_client_with_data().await;

    let request = tonic::Request::new(ListUsersRequest {});

    let response = client.list_users(request).await.unwrap();
    let users = response.into_inner().users;

    assert_eq!(users.len(), 2);
    assert_eq!(users[0].id, 2);
    assert_eq!(users[1].id, 1);
}

#[tokio::test]
async fn test_list_users_empty_grpc() {
    let (mut client, _handle) = setup_user_grpc_client().await;

    let request = tonic::Request::new(ListUsersRequest {});

    let response = client.list_users(request).await.unwrap();
    let users = response.into_inner().users;

    assert_eq!(users.len(), 0);
}

#[tokio::test]
async fn test_update_user_grpc() {
    let (mut client, _handle) = setup_user_grpc_client_with_data().await;

    let request = tonic::Request::new(UpdateUserRequest {
        id: 1,
        name: Some("Updated Name".to_string()),
        email: Some("updated@example.com".to_string()),
    });

    let response = client.update_user(request).await.unwrap();
    let user = response.into_inner().user.unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "Updated Name");
    assert_eq!(user.email, "updated@example.com");
}

#[tokio::test]
async fn test_update_user_partial_grpc() {
    let (mut client, _handle) = setup_user_grpc_client_with_data().await;

    let request = tonic::Request::new(UpdateUserRequest {
        id: 1,
        name: Some("New Name".to_string()),
        email: None,
    });

    let response = client.update_user(request).await.unwrap();
    let user = response.into_inner().user.unwrap();

    assert_eq!(user.id, 1);
    assert_eq!(user.name, "New Name");
    assert_eq!(user.email, "john@example.com");
}

#[tokio::test]
async fn test_update_user_not_found_grpc() {
    let (mut client, _handle) = setup_user_grpc_client().await;

    let request = tonic::Request::new(UpdateUserRequest {
        id: 999,
        name: Some("Updated".to_string()),
        email: None,
    });

    let result = client.update_user(request).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_user_grpc() {
    let (mut client, _handle) = setup_user_grpc_client_with_data().await;

    let request = tonic::Request::new(DeleteUserRequest { id: 1 });

    let response = client.delete_user(request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.success, true);

    let get_request = tonic::Request::new(GetUserRequest { id: 1 });
    let get_result = client.get_user(get_request).await;
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_delete_user_not_found_grpc() {
    let (mut client, _handle) = setup_user_grpc_client().await;

    let request = tonic::Request::new(DeleteUserRequest { id: 999 });

    let response = client.delete_user(request).await.unwrap();
    let result = response.into_inner();

    assert_eq!(result.success, false);
}
