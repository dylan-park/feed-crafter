mod api;
mod common;
mod web;

use api::*;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
use common::*;
use dotenvy::dotenv;
use log::{debug, info};
use std::{
    env, fs,
    sync::{Arc, Mutex},
};
use tokio::{
    net::TcpListener,
    time::{Duration, interval},
};
use tower_http::services::ServeDir;
use web::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    // Ensure the ./feed directory exists
    fs::create_dir_all("./feed").expect("Failed to create ./feed directory");

    // Initialize or load the RSS feed
    let channel = initialize_feed(&RealFileSystem);
    let app_state = AppState {
        channel: Arc::new(Mutex::new(channel)),
    };

    // Start the cleanup timer
    start_cleanup_timer(app_state.clone());

    // Build our application with routes
    let app = Router::new()
        // Web interface routes
        .route("/", get(index))
        .route("/feed.xml", get(serve_file))
        .route("/add", get(add_item_form))
        .route("/add", post(web_add_item))
        .route("/delete/{id}", post(web_delete_item))
        .route("/edit/{id}", get(edit_item_form))
        .route("/edit/{id}", post(web_edit_item))
        .route("/health", get(health_check))
        // API routes
        .route("/api/items", get(api_get_items))
        .route("/api/items", post(api_add_item))
        .route("/api/items/{id}", delete(api_delete_item))
        .route("/api/items/{id}", put(api_edit_item))
        .nest_service("/static", ServeDir::new("static"))
        .with_state(app_state);

    // Start the server
    let address = env::var("SERVER_ADDRESS").expect("Expected a server address in the environment");
    let port = env::var("SERVER_PORT").expect("Expected a server port in the environment");
    let listener = TcpListener::bind(format!("{}:{}", address, port))
        .await
        .expect("Failed to bind to address");

    info!("Server running on http://{}:{}", address, port);
    axum::serve(listener, app).await.unwrap();
}

pub fn start_cleanup_timer(state: AppState) {
    let cleanup_interval_seconds = env::var("CLEANUP_INTERVAL_SECONDS")
        .unwrap_or_else(|_| "3600".to_string()) // Default to 1 hour
        .parse::<u64>()
        .unwrap_or(3600);

    tokio::spawn(async move {
        let mut interval_timer = interval(Duration::from_secs(cleanup_interval_seconds));

        // Skip the first tick (which fires immediately)
        interval_timer.tick().await;

        loop {
            interval_timer.tick().await;

            let removed_count = cleanup_old_items(&state, &RealFileSystem);
            if removed_count > 0 {
                debug!("Periodic cleanup removed {} items", removed_count);
            } else {
                debug!("Periodic cleanup - no items to remove");
            }
        }
    });
}
