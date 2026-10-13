use axum::{routing::get, Router};

// Write the `health` handler returning "ok"
async fn health() -> &'static str {
    "ok"
}

#[tokio::main]
async fn main() {
    // Register GET /health on the router
    let app = Router::new().route("/health", get(health));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
