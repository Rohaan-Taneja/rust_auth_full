use std::sync::Arc;

use axum::{Extension, Router};
use tower_http::trace::TraceLayer;

use crate::{AppState, handler::auth::auth_handler};

pub fn create_router(app_state : Arc<AppState>)->Router{

    let api_route = Router::new()
        .nest("/users", auth_handler())
        .layer(TraceLayer::new_for_http()) //see difference ki ky aa rha hai , with or without me 
        .layer(Extension(app_state));


    Router::new().nest("/api", api_route)


}