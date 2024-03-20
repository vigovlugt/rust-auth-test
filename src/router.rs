use crate::{
    auth::handlers::{handle_auth_callback, handle_auth_url},
    users::handlers::users_me_handler,
    AppState,
};
use axum::{routing::get, Router};
use std::sync::Arc;

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/users/me", get(users_me_handler))
        .route("/login/:provider", get(handle_auth_url))
        .route("/login/:provider/callback", get(handle_auth_callback))
        .with_state(state)
}
