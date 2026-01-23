use std::sync::Arc;

use axum::{Extension, Router, middleware};
use tower_http::trace::TraceLayer;

use crate::{
    AppState,
    handler::{auth::auth_handler, users::users_handler},
    middleware::auth,
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_route = Router::new()
        .nest("/auth", auth_handler())
        .nest("/users", users_handler().layer(middleware::from_fn(auth))) // routes which will have auth middleware protection
        .layer(TraceLayer::new_for_http()) //see difference ki ky aa rha hai , with or without me
        .layer(Extension(app_state));

    Router::new().nest("/api", api_route)
}
