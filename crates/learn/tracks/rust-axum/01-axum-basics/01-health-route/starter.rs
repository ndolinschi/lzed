use axum::{routing::get, Router};

// Write the `health` handler returning "ok"


#[tokio::main]
async fn main() {
    // Register GET /health on the router
    let app = Router::new();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
