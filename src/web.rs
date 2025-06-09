use crate::common::*;
use askama::Template;
use axum::{
    Json,
    body::Body,
    extract::{Form, Path, State},
    http::{Response, StatusCode},
    response::{Html, IntoResponse, Redirect},
};
use log::info;
use rss::{Channel, Item};
use serde::Deserialize;
use std::{fs, path::Path as StdPath};

// Templates
#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate {
    channel: Channel,
}

#[derive(Template)]
#[template(path = "add_item.html")]
struct AddItemTemplate {}

#[derive(Template)]
#[template(path = "edit_item.html")]
struct EditItemTemplate {
    item: Item,
    item_id: String,
}

// Form data structures
#[derive(Deserialize)]
pub struct NewItemForm {
    title: String,
    description: Option<String>,
    link: Option<String>,
}

#[derive(Deserialize)]
pub struct EditItemForm {
    title: String,
    description: Option<String>,
    link: Option<String>,
}

// Health Check
#[derive(serde::Serialize)]
struct HealthStatus {
    status: String,
    timestamp: i64,
    checks: std::collections::HashMap<String, CheckResult>,
}

#[derive(serde::Serialize)]
struct CheckResult {
    status: String,
    message: Option<String>,
}

pub async fn index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let channel = state.channel.lock().unwrap().clone();
    let template = IndexTemplate { channel };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn serve_file() -> impl IntoResponse {
    let path = StdPath::new("./feed/feed.xml");

    match fs::read(path) {
        Ok(contents) => {
            let body = Body::from(contents);
            Response::builder()
                .header("Content-Type", "application/rss+xml")
                .body(body)
                .unwrap()
        }
        Err(_) => (
            StatusCode::NOT_FOUND,
            "File not found or couldn't be read".to_string(),
        )
            .into_response(),
    }
}

pub async fn add_item_form() -> Result<Html<String>, StatusCode> {
    let template = AddItemTemplate {};
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn edit_item_form(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<Html<String>, StatusCode> {
    let channel = state.channel.lock().unwrap();

    // Find the item with the matching GUID
    let item = channel
        .items()
        .iter()
        .find(|item| item.guid().map(|g| g.value() == item_id).unwrap_or(false))
        .ok_or(StatusCode::NOT_FOUND)?
        .clone();

    let template = EditItemTemplate { item, item_id };
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn web_add_item(
    State(state): State<AppState>,
    Form(form): Form<NewItemForm>,
) -> Result<Redirect, StatusCode> {
    if form.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let item = create_item(
        form.title,
        form.description.filter(|s| !s.trim().is_empty()),
        form.link.filter(|s| !s.trim().is_empty()),
    );

    add_item(axum::extract::State(state), item.clone());
    info!("Item added successfully: {}", item.guid().unwrap().value);

    Ok(Redirect::to("/"))
}

pub async fn web_delete_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    delete_item(
        axum::extract::State(state),
        axum::extract::Path(item_id.clone()),
    );
    info!("Item deleted successfully: {}", item_id);

    Ok(Redirect::to("/"))
}

pub async fn web_edit_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Form(form): Form<EditItemForm>,
) -> Result<Redirect, StatusCode> {
    if form.title.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let item = edit_item(
        axum::extract::State(state),
        axum::extract::Path(item_id),
        form.title,
        form.description,
        form.link,
    );

    if item.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    info!(
        "Item edited successfully: {}",
        item.unwrap().guid().unwrap().value
    );

    Ok(Redirect::to("/"))
}

pub async fn health_check() -> impl IntoResponse {
    let mut checks = std::collections::HashMap::new();
    let mut overall_healthy = true;

    // File existence check
    if StdPath::new("./feed/feed.xml").exists() {
        checks.insert(
            "feed_file".to_string(),
            CheckResult {
                status: "healthy".to_string(),
                message: None,
            },
        );
    } else {
        checks.insert(
            "feed_file".to_string(),
            CheckResult {
                status: "unhealthy".to_string(),
                message: Some("feed.xml not found".to_string()),
            },
        );
        overall_healthy = false;
    }

    // Directory permissions check
    let feed_dir = StdPath::new("./feed");
    if feed_dir.exists() {
        let test_file = feed_dir.join(".health_temp");
        match fs::write(&test_file, "test") {
            Ok(_) => {
                let _ = fs::remove_file(&test_file);
                checks.insert(
                    "directory_writable".to_string(),
                    CheckResult {
                        status: "healthy".to_string(),
                        message: None,
                    },
                );
            }
            Err(e) => {
                checks.insert(
                    "directory_writable".to_string(),
                    CheckResult {
                        status: "unhealthy".to_string(),
                        message: Some(format!("Cannot write to feed directory: {}", e)),
                    },
                );
                overall_healthy = false;
            }
        }
    } else {
        checks.insert(
            "directory_writable".to_string(),
            CheckResult {
                status: "unhealthy".to_string(),
                message: Some("Feed directory does not exist".to_string()),
            },
        );
        overall_healthy = false;
    }

    let health_status = HealthStatus {
        status: if overall_healthy {
            "healthy".to_string()
        } else {
            "unhealthy".to_string()
        },
        timestamp: chrono::Utc::now().timestamp(),
        checks,
    };

    let status_code = if overall_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status_code, Json(health_status))
}
