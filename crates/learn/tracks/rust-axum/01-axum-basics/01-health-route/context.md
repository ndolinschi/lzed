In Axum, a `Router` maps paths to handlers. A handler is just an `async fn` whose return type implements `IntoResponse` — `&'static str` works.

```rust
let app = Router::new().route("/", get(root));
```

This exercise is checked statically (structure of your code), so no Rust toolchain download is needed.
