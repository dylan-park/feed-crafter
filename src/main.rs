mod api;
mod common;
mod web;

use api::*;
use axum::{
    Router,
    routing::{delete, get, post},
};
use common::*;
use dotenvy::dotenv;
use std::{
    env, fs,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use web::*;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // Ensure the ./feed directory exists
    fs::create_dir_all("./feed").expect("Failed to create ./feed directory");

    // Initialize or load the RSS feed
    let channel = initialize_feed();
    let app_state = AppState {
        channel: Arc::new(Mutex::new(channel)),
    };

    // Build our application with routes
    let app = Router::new()
        // Web interface routes
        .route("/", get(index))
        .route("/feed.xml", get(serve_file))
        .route("/add", get(add_item_form))
        .route("/add", post(add_item))
        .route("/delete/{id}", post(delete_item))
        .route("/health", get(health_check))
        // API routes
        .route("/api/items", get(api_get_items))
        .route("/api/items", post(api_add_item))
        .route("/api/items/{id}", delete(api_delete_item))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Start the server
    let address = env::var("SERVER_ADDRESS").expect("Expected a server address in the environment");
    let port = env::var("SERVER_PORT").expect("Expected a server port in the environment");
    let listener = TcpListener::bind(format!("{}:{}", address, port))
        .await
        .expect("Failed to bind to address");

    println!("Server running on http://{}:{}", address, port);
    axum::serve(listener, app).await.unwrap();
}
