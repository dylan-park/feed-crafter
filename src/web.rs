use crate::common::*;
use askama::Template;
use axum::{
    body::Body,
    extract::{Form, Path, State},
    http::{Response, StatusCode},
    response::{Html, IntoResponse, Redirect},
};
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

// Form data structures
#[derive(Deserialize)]
pub struct NewItemForm {
    title: String,
    description: Option<String>,
    link: Option<String>,
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

pub async fn add_item(
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

    {
        let mut channel = state.channel.lock().unwrap();
        let mut items = channel.items().to_vec();
        items.insert(0, item);
        channel.set_items(items);
        channel.set_last_build_date(chrono::Utc::now().to_rfc2822());

        // Save to file
        write_channel(&channel, None);
    }

    Ok(Redirect::to("/"))
}

pub async fn delete_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Result<Redirect, StatusCode> {
    {
        let mut channel = state.channel.lock().unwrap();
        let items: Vec<Item> = channel
            .items()
            .iter()
            .filter(|item| item.guid().map(|g| g.value() != item_id).unwrap_or(true))
            .cloned()
            .collect();

        channel.set_items(items);
        channel.set_last_build_date(chrono::Utc::now().to_rfc2822());

        // Save to file
        write_channel(&channel, None);
    }

    Ok(Redirect::to("/"))
}

pub async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}
