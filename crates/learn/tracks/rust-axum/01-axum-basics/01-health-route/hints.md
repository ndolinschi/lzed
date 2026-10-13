`async fn health() -> &'static str { "ok" }`

---

Chain `.route("/health", get(health))` onto `Router::new()`.
