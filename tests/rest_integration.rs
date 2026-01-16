mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

#[tokio::test]
async fn test_create_task_rest() {
    let pool = common::setup_test_pool().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/tasks")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Test Task",
                        "description": "Test Description"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(task["title"], "Test Task");
    assert_eq!(task["description"], "Test Description");
    assert_eq!(task["completed"], false);
    assert!(task["id"].as_i64().unwrap() > 0);
}

#[tokio::test]
async fn test_get_task_rest() {
    let pool = common::setup_test_pool_with_data().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(task["id"], 1);
    assert_eq!(task["title"], "Test Task 1");
    assert_eq!(task["description"], "Description 1");
    assert_eq!(task["completed"], false);
}

#[tokio::test]
async fn test_get_task_not_found_rest() {
    let pool = common::setup_test_pool().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let status = response.status();
    if status != StatusCode::NOT_FOUND {
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_str = String::from_utf8_lossy(&body);
        panic!("Expected 404, got {}. Body: {}", status, body_str);
    }
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_tasks_rest() {
    let pool = common::setup_test_pool_with_data().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["id"], 2);
    assert_eq!(tasks[1]["id"], 1);
}

#[tokio::test]
async fn test_list_tasks_empty_rest() {
    let pool = common::setup_test_pool().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let tasks: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();

    assert_eq!(tasks.len(), 0);
}

#[tokio::test]
async fn test_update_task_rest() {
    let pool = common::setup_test_pool_with_data().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/tasks/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Updated Task",
                        "description": "Updated Description",
                        "completed": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(task["id"], 1);
    assert_eq!(task["title"], "Updated Task");
    assert_eq!(task["description"], "Updated Description");
    assert_eq!(task["completed"], true);
}

#[tokio::test]
async fn test_update_task_partial_rest() {
    let pool = common::setup_test_pool_with_data().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/tasks/1")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "completed": true
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let task: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(task["id"], 1);
    assert_eq!(task["title"], "Test Task 1");
    assert_eq!(task["description"], "Description 1");
    assert_eq!(task["completed"], true);
}

#[tokio::test]
async fn test_update_task_not_found_rest() {
    let pool = common::setup_test_pool().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/tasks/999")
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "title": "Updated"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_task_rest() {
    let pool = common::setup_test_pool_with_data().await;

    let app = rust_grpc_sqlite::rest_server::create_router(pool.clone());
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/tasks/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let app2 = rust_grpc_sqlite::rest_server::create_router(pool.clone());
    let response2 = app2
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/tasks/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response2.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_task_not_found_rest() {
    let pool = common::setup_test_pool().await;
    let app = rust_grpc_sqlite::rest_server::create_router(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/tasks/999")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
