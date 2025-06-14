use axum::extract::{Path as AxumPath, State};
use log::{debug, info, warn};
use rss::{Channel, ChannelBuilder, Guid, Item, ItemBuilder};
use std::sync::{Arc, Mutex};
use std::{
    env,
    fs::{File, write},
    io::{BufReader, Read},
    path::Path,
};
use uuid::Uuid;

pub trait FileSystem {
    type Reader: Read;

    fn exists(&self, path: &str) -> bool;
    fn open(&self, path: &str) -> Result<Self::Reader, std::io::Error>;
    fn write(&self, path: &str, contents: &str) -> Result<(), std::io::Error>;
}

// Real filesystem implementation
pub struct RealFileSystem;

impl FileSystem for RealFileSystem {
    type Reader = File;

    fn exists(&self, path: &str) -> bool {
        Path::new(path).exists()
    }

    fn open(&self, path: &str) -> Result<Self::Reader, std::io::Error> {
        File::open(path)
    }

    fn write(&self, path: &str, contents: &str) -> Result<(), std::io::Error> {
        write(path, contents)
    }
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub channel: Arc<Mutex<Channel>>,
}

pub fn initialize_feed<F: FileSystem>(fs: &F) -> Channel
where
    F::Reader: Read,
{
    if fs.exists("./feed/feed.xml") {
        info!("Feed found on disk, reading...");
        let file = fs.open("./feed/feed.xml").expect("Error opening feed.xml");
        let reader = BufReader::new(file);
        let channel = Channel::read_from(reader).expect("Error reading feed into Channel");
        info!("Feed successfully read from disk");
        channel
    } else {
        info!("No feed found on disk, creating based on environment variables");
        let channel = create_feed();
        write_channel(&channel, None, fs);
        info!("Feed successfully created and written to disk");
        channel
    }
}

pub fn create_feed() -> Channel {
    ChannelBuilder::default()
        .title(env::var("CHANNEL_TITLE").expect("Expected a channel title in the environment"))
        .link(env::var("CHANNEL_LINK").expect("Expected a channel link in the environment"))
        .description(
            env::var("CHANNEL_DESCRIPTION")
                .expect("Expected a channel description in the environment"),
        )
        .last_build_date(chrono::Utc::now().to_rfc2822())
        .build()
}

pub fn create_item(title: String, description: Option<String>, link: Option<String>) -> Item {
    let mut binding = ItemBuilder::default();
    let mut builder = binding
        .title(Some(title))
        .guid(Some(rss::Guid {
            value: Uuid::new_v4().to_string(),
            permalink: false,
        }))
        .pub_date(Some(chrono::Utc::now().to_rfc2822()));

    if let Some(description) = description {
        builder = builder.description(Some(description));
    }
    if let Some(link) = link {
        builder = builder.link(Some(link));
    }

    let item = builder.build();
    info!(
        "Item Created:\n{{\n\t\"title\": \"{}\"\n\t\"description\": \"{}\"\n\t\"link\": \"{}\"\n}}",
        item.clone().title.unwrap(),
        item.clone().description.unwrap_or_default(),
        item.clone().link.unwrap_or_default()
    );
    item
}

pub fn write_channel<F: FileSystem>(channel: &Channel, path: Option<&str>, fs: &F) {
    let rss_content = channel.to_string();
    let file_path = path.unwrap_or("./feed/feed.xml");
    fs.write(file_path, &rss_content)
        .expect("Failed to write RSS feed to file");
    info!("Feed written successfully");
}

pub fn add_item(State(state): State<AppState>, item: Item) {
    let mut channel = state.channel.lock().unwrap();
    let mut items = channel.items().to_vec();
    items.insert(0, item);
    channel.set_items(items);
    channel.set_last_build_date(chrono::Utc::now().to_rfc2822());

    // Save to file
    write_channel(&channel, None, &RealFileSystem);
}

pub fn delete_item(
    State(state): State<AppState>,
    AxumPath(item_id): AxumPath<String>,
) -> Option<Guid> {
    let mut return_item_id: Option<Guid> = None;

    {
        let mut channel = state.channel.lock().unwrap();

        let items: Vec<Item> = channel
            .items()
            .iter()
            .filter(|item| {
                let matches = item.guid().map(|g| g.value() == item_id).unwrap_or(false);
                if matches {
                    return_item_id = item.guid().cloned();
                }
                !matches
            })
            .cloned()
            .collect();

        if return_item_id.is_some() {
            channel.set_items(items);
            channel.set_last_build_date(chrono::Utc::now().to_rfc2822());
            write_channel(&channel, None, &RealFileSystem);
        }
    }
    return_item_id
}

pub fn edit_item(
    State(state): State<AppState>,
    AxumPath(item_id): AxumPath<String>,
    title: String,
    description: Option<String>,
    link: Option<String>,
) -> Option<Item> {
    let mut return_item: Option<Item> = None;

    {
        let mut channel = state.channel.lock().unwrap();

        let items: Vec<Item> = channel
            .items()
            .iter()
            .map(|item| {
                let matches = item.guid().map(|g| g.value() == item_id).unwrap_or(false);
                if matches {
                    // Create updated item
                    let updated_item = create_item(
                        title.clone(),
                        description.clone().filter(|s| !s.trim().is_empty()),
                        link.clone().filter(|s| !s.trim().is_empty()),
                    );
                    return_item = Some(updated_item.clone());
                    updated_item
                } else {
                    item.clone()
                }
            })
            .collect();

        if return_item.is_some() {
            channel.set_items(items);
            channel.set_last_build_date(chrono::Utc::now().to_rfc2822());
            write_channel(&channel, None, &RealFileSystem);
        }
    }
    return_item
}

pub fn cleanup_old_items<F: FileSystem>(state: &AppState, fs: &F) -> usize {
    let max_age_seconds = match env::var("MAX_ITEM_AGE_SECONDS") {
        Ok(val) => match val.parse::<u64>() {
            Ok(seconds) => seconds,
            Err(_) => {
                warn!(
                    "Invalid MAX_ITEM_AGE_SECONDS value: '{}', skipping cleanup",
                    val
                );
                return 0;
            }
        },
        Err(_) => {
            debug!("MAX_ITEM_AGE_SECONDS not set, skipping cleanup");
            return 0;
        }
    };

    if max_age_seconds == 0 {
        debug!("MAX_ITEM_AGE_SECONDS is 0, items will be kept indefinitely");
        return 0;
    }

    let cutoff_date = chrono::Utc::now() - chrono::Duration::seconds(max_age_seconds as i64);

    let items_removed = {
        let mut channel = state.channel.lock().unwrap();
        let original_count = channel.items().len();

        let items: Vec<Item> = channel
            .items()
            .iter()
            .filter(|item| {
                if let Some(pub_date_str) = item.pub_date() {
                    match chrono::DateTime::parse_from_rfc2822(pub_date_str) {
                        Ok(pub_date) => {
                            let is_old = pub_date.with_timezone(&chrono::Utc) < cutoff_date;
                            if is_old {
                                info!(
                                    "Removing old item: '{}'",
                                    item.title().unwrap_or("Untitled")
                                );
                            }
                            !is_old
                        }
                        Err(_) => {
                            warn!(
                                "Invalid pub_date format for item: '{}'",
                                item.title().unwrap_or("Untitled")
                            );
                            true // Keep items with invalid dates
                        }
                    }
                } else {
                    warn!(
                        "Item has no pub_date: '{}'",
                        item.title().unwrap_or("Untitled")
                    );
                    true // Keep items without pub_date
                }
            })
            .cloned()
            .collect();

        let removed_count = original_count - items.len();

        if removed_count > 0 {
            channel.set_items(items);
            channel.set_last_build_date(chrono::Utc::now().to_rfc2822());
            write_channel(&channel, None, fs);
            info!("Cleaned up {} old items from feed", removed_count);
        }

        removed_count
    };

    items_removed
}
