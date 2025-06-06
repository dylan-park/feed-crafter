use crate::common::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use rss::Item;
use serde::Deserialize;

// API data structures
#[derive(Deserialize)]
pub struct ApiNewItem {
    title: String,
    description: String,
    link: Option<String>,
}

#[derive(serde::Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[derive(serde::Serialize)]
pub struct ApiItem {
    id: String,
    title: String,
    description: String,
    link: Option<String>,
    pub_date: Option<String>,
}

// API route handlers
pub async fn api_get_items(State(state): State<AppState>) -> Json<ApiResponse<Vec<ApiItem>>> {
    let channel = state.channel.lock().unwrap();
    let items: Vec<ApiItem> = channel
        .items()
        .iter()
        .map(|item| ApiItem {
            id: item
                .guid()
                .map(|g| g.value().to_string())
                .unwrap_or_default(),
            title: item.title().unwrap_or("Untitled").to_string(),
            description: item.description().unwrap_or("No description").to_string(),
            link: item.link().map(|s| s.to_string()),
            pub_date: item.pub_date().map(|s| s.to_string()),
        })
        .collect();

    Json(ApiResponse {
        success: true,
        data: Some(items),
        message: "Items retrieved successfully".to_string(),
    })
}

pub async fn api_add_item(
    State(state): State<AppState>,
    Json(payload): Json<ApiNewItem>,
) -> Result<Json<ApiResponse<ApiItem>>, StatusCode> {
    if payload.title.trim().is_empty() || payload.description.trim().is_empty() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: "Title and description are required".to_string(),
        }));
    }

    let item = create_item(
        payload.title.clone(),
        payload.description.clone(),
        payload.link.clone().filter(|s| !s.trim().is_empty()),
    );

    let api_item = ApiItem {
        id: item
            .guid()
            .map(|g| g.value().to_string())
            .unwrap_or_default(),
        title: payload.title,
        description: payload.description,
        link: payload.link,
        pub_date: item.pub_date().map(|s| s.to_string()),
    };

    {
        let mut channel = state.channel.lock().unwrap();
        let mut items = channel.items().to_vec();
        items.insert(0, item);
        channel.set_items(items);

        // Save to file
        write_channel(&channel, None);
    }

    Ok(Json(ApiResponse {
        success: true,
        data: Some(api_item),
        message: "Item added successfully".to_string(),
    }))
}

pub async fn api_delete_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
) -> Json<ApiResponse<()>> {
    let mut found = false;

    {
        let mut channel = state.channel.lock().unwrap();

        let items: Vec<Item> = channel
            .items()
            .iter()
            .filter(|item| {
                let matches = item.guid().map(|g| g.value() == item_id).unwrap_or(false);
                if matches {
                    found = true;
                }
                !matches
            })
            .cloned()
            .collect();

        if found {
            channel.set_items(items);
            write_channel(&channel, None);
        }
    }

    if found {
        Json(ApiResponse {
            success: true,
            data: Some(()),
            message: "Item deleted successfully".to_string(),
        })
    } else {
        Json(ApiResponse {
            success: false,
            data: None,
            message: "Item not found".to_string(),
        })
    }
}
