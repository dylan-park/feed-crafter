use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use std::sync::{Arc, Mutex};
use std::{env, fs::File, io::BufReader};
use std::{fs, path::Path as StdPath};
use uuid::Uuid;

// Application state
#[derive(Clone)]
pub struct AppState {
    pub channel: Arc<Mutex<Channel>>,
}

pub fn initialize_feed() -> Channel {
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

    builder.build()
}

pub fn write_channel(channel: &Channel, path: Option<&StdPath>) {
    let rss_content = channel.to_string();
    let file_path = path.unwrap_or_else(|| StdPath::new("./feed/feed.xml"));
    fs::write(file_path, &rss_content).expect("Failed to write RSS feed to file");
}
