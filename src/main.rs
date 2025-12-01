mod config;
mod db;
mod dtos;
mod errors;
mod models;
mod schema;
use axum::{Router, http::{HeaderValue, Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}}, serve::Listener};
use diesel::{
    Connection, PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use tower_http::cors::CorsLayer;

use crate::config::Config;
use dotenvy::dotenv;
use std::env;

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
        .max_size(10)
        .build(manager)
        .expect("failer to create database pool");

    // creating app state
    // let app = AppState { db: pool };

    // cors setup
    // let cors = CorsLayer::new()
    //     .allow_origin("*".parse::<HeaderValue>().unwrap())
    //     .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
    //     .allow_credentials(true)
    //     .allow_methods([Method::GET, Method::POST, Method::PUT]);

    // building the router
    let app = Router::new();

    // server setup
    let port = env::var("PORT").unwrap_or_else( |_| "8080".to_string());
    let host = env::var("HOST").unwrap_or_else( |_| "127.0.0.1".to_string());
    let addr = format!("{}:{}", host, port);

    tracing::info!("starting the server at  {}", port);

    // now os will assign specific host and port to our app
    // let listener = tokio::net::TcpListener::bind(addr)
    //     .await
    //     .expect("failed to start server");

    // axum::serve(listener, app).await.expect("failed to create axum server");

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

}

