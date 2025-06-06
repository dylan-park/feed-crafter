use askama::Template;
use axum::{
    Router,
    body::Body,
    extract::{Form, Path, State},
    http::{Response, StatusCode},
    response::{Html, IntoResponse, Json, Redirect},
    routing::{delete, get, post},
};
use dotenvy::dotenv;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use serde::Deserialize;
use std::{
    env,
    fs::{self, File},
    io::BufReader,
    path::Path as StdPath,
    sync::{Arc, Mutex},
};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use uuid::Uuid;

// Application state
#[derive(Clone)]
struct AppState {
    channel: Arc<Mutex<Channel>>,
}

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
struct NewItemForm {
    title: String,
    description: String,
    link: Option<String>,
}

// API data structures
#[derive(Deserialize)]
struct ApiNewItem {
    title: String,
    description: String,
    link: Option<String>,
}

#[derive(serde::Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: String,
}

#[derive(serde::Serialize)]
struct ApiItem {
    id: String,
    title: String,
    description: String,
    link: Option<String>,
    pub_date: Option<String>,
}

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

fn initialize_feed() -> Channel {
    if StdPath::new("./feed/feed.xml").exists() {
        let file = File::open("./feed/feed.xml").expect("Error opening feed.xml");
        let reader = BufReader::new(file);
        Channel::read_from(reader).expect("Error reading feed into Channel")
    } else {
        let channel = create_feed();
        write_channel(&channel, None);
        channel
    }
}

fn create_feed() -> Channel {
    ChannelBuilder::default()
        .title(env::var("CHANNEL_TITLE").expect("Expected a channel title in the environment"))
        .link(env::var("CHANNEL_LINK").expect("Expected a channel link in the environment"))
        .description(
            env::var("CHANNEL_DESCRIPTION")
                .expect("Expected a channel description in the environment"),
        )
        .build()
}

fn create_item(title: String, description: String, link: Option<String>) -> Item {
    let mut binding = ItemBuilder::default();
    let mut builder = binding
        .title(Some(title))
        .description(Some(description))
        .guid(Some(rss::Guid {
            value: Uuid::new_v4().to_string(),
            permalink: false,
        }))
        .pub_date(Some(chrono::Utc::now().to_rfc2822()));

    if let Some(link) = link {
        builder = builder.link(Some(link));
    }

    builder.build()
}

fn write_channel(channel: &Channel, path: Option<&StdPath>) {
    let rss_content = channel.to_string();
    let file_path = path.unwrap_or_else(|| StdPath::new("./feed/feed.xml"));
    fs::write(file_path, &rss_content).expect("Failed to write RSS feed to file");
}

// Route handlers
async fn index(State(state): State<AppState>) -> Result<Html<String>, StatusCode> {
    let channel = state.channel.lock().unwrap().clone();
    let template = IndexTemplate { channel };

    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn serve_file() -> impl IntoResponse {
    let path = StdPath::new("./feed/feed.xml");

    match fs::read(&path) {
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

async fn add_item_form() -> Result<Html<String>, StatusCode> {
    let template = AddItemTemplate {};
    match template.render() {
        Ok(html) => Ok(Html(html)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn add_item(
    State(state): State<AppState>,
    Form(form): Form<NewItemForm>,
) -> Result<Redirect, StatusCode> {
    if form.title.trim().is_empty() || form.description.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let item = create_item(
        form.title,
        form.description,
        form.link.filter(|s| !s.trim().is_empty()),
    );

    {
        let mut channel = state.channel.lock().unwrap();
        let mut items = channel.items().to_vec();
        items.insert(0, item);
        channel.set_items(items);

        // Save to file
        write_channel(&channel, None);
    }

    Ok(Redirect::to("/"))
}

async fn delete_item(
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

        // Save to file
        write_channel(&channel, None);
    }

    Ok(Redirect::to("/"))
}

async fn health_check() -> impl IntoResponse {
    StatusCode::OK
}

// API route handlers
async fn api_get_items(State(state): State<AppState>) -> Json<ApiResponse<Vec<ApiItem>>> {
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

async fn api_add_item(
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

async fn api_delete_item(
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
