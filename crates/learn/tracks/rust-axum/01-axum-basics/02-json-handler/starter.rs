use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

struct CreateTask {
    title: String,
}

struct Task {
    id: u64,
    title: String,
}

// Write `create_task`


#[tokio::main]
async fn main() {
    let app = Router::new();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
