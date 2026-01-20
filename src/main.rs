#![allow(warnings)]
#![allow(clippy::to_string_trait_impl)]
mod config;
mod db;
mod dtos;
mod errors;
mod handler;
mod mail;
mod middleware;
mod models;
mod schema;
mod utils;
use axum::{
    Extension, Router,
    http::{
        HeaderValue, Method,
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    },
    serve::Listener,
};
mod routes;
use diesel::{
    Connection, PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use tower_http::cors::CorsLayer;

use crate::{config::Config, routes::create_router};
use dotenvy::dotenv;
use std::{clone, env, sync::Arc};

// type for data base pool/collection_of_db_connection
pub type DbPool = Pool<ConnectionManager<PgConnection>>;

// application state
#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
}

#[tokio::main]
async fn main() {
    // loading env variables
    dotenv().ok();

    // see what it is doing
    // initializing tracing and logging
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    // getting the db and jwt env variables
    let config: Config = Config::init();

    // creating pool of db connection
    let manager = ConnectionManager::<PgConnection>::new(config.database_url);
    let pool = Pool::builder()
        .max_size(3)
        .build(manager)
        .expect("failer to create database pool");

    // cors setup
    let cors = CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        // .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT]);

    // creating app state
    let app_state = AppState { db: pool };

    let a = app_state.clone();
    // building the router
    let app = create_router(Arc::new(app_state.clone())).layer(cors.clone());

    // server setup
    let port = env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port);

    println!("this is the addr {}", addr);
    // wrting info in console
    tracing::info!("starting the server at  {}", addr);

    // now os will assign specific host and port to our app
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to start server");

    axum::serve(listener, app).await.unwrap();
}
