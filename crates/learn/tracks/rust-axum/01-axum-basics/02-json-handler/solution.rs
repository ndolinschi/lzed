use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CreateTask {
    title: String,
}

#[derive(Serialize)]
struct Task {
    id: u64,
    title: String,
}

// Write `create_task`
async fn create_task(Json(input): Json<CreateTask>) -> Json<Task> {
    Json(Task { id: 1, title: input.title })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/tasks", post(create_task));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
