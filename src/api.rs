use crate::common::*;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

// API data structures
#[derive(Deserialize)]
pub struct ApiNewItem {
    title: String,
    description: Option<String>,
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
    description: Option<String>,
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
            description: item.description().map(|s| s.to_string()),
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
    if payload.title.trim().is_empty() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: "Title is required".to_string(),
        }));
    }

    let item = create_item(
        payload.title.clone(),
        payload.description.clone().filter(|s| !s.trim().is_empty()),
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

    add_item(axum::extract::State(state), item);

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
    let item_id = delete_item(axum::extract::State(state), axum::extract::Path(item_id));

    if item_id.is_some() {
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

pub async fn api_edit_item(
    State(state): State<AppState>,
    Path(item_id): Path<String>,
    Json(payload): Json<ApiNewItem>,
) -> Result<Json<ApiResponse<ApiItem>>, StatusCode> {
    if payload.title.trim().is_empty() {
        return Ok(Json(ApiResponse {
            success: false,
            data: None,
            message: "Title is required".to_string(),
        }));
    }

    let mut found_item: Option<ApiItem> = None;

    {
        let updated_item = edit_item(
            axum::extract::State(state),
            axum::extract::Path(item_id),
            payload.title.clone(),
            payload.description.clone(),
            payload.link.clone(),
        );
        if updated_item.is_some() {
            // Store the API representation for response
            found_item = Some(ApiItem {
                id: updated_item
                    .as_ref()
                    .unwrap()
                    .guid()
                    .map(|g| g.value().to_string())
                    .unwrap_or_default(),
                title: payload.title.clone(),
                description: payload.description.clone(),
                link: payload.link.clone(),
                pub_date: updated_item
                    .as_ref()
                    .unwrap()
                    .pub_date()
                    .map(|s| s.to_string()),
            });
        }

        if let Some(api_item) = found_item {
            Ok(Json(ApiResponse {
                success: true,
                data: Some(api_item),
                message: "Item updated successfully".to_string(),
            }))
        } else {
            Ok(Json(ApiResponse {
                success: false,
                data: None,
                message: "Item not found".to_string(),
            }))
        }
    }
}
