use dotenvy::dotenv;
use rss::{Channel, ChannelBuilder, Item, ItemBuilder};
use std::env;
use std::fs::{self, File};
use std::io::BufReader;
use std::path::Path;

fn main() {
    dotenv().ok();

    // RSS Feed file exists
    if Path::new("feed.xml").exists() {
        let file = File::open("feed.xml").expect("Error opening feed.xml");
        let reader = BufReader::new(file);
        let channel = Channel::read_from(reader).expect("Error reading feed into Channel");

        println!("{}", channel);
    // RSS Feed file does not exist
    } else {
        let mut channel = create_feed();
        let item = create_item(
            "New Blog Post".to_string(),
            "This is the latest blog post.".to_string(),
        );
        prepend_item_to_channel(&mut channel, item);
        write_channel(channel, None);
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

fn create_item(title: String, description: String) -> Item {
    ItemBuilder::default()
        .title(Some(title))
        .description(Some(description))
        .build()
}

fn prepend_item_to_channel(channel: &mut Channel, item: Item) {
    let mut items = channel.items().to_vec();
    items.insert(0, item);
    channel.set_items(items);
}

fn write_channel(channel: Channel, path: Option<&Path>) {
    let rss_content = channel.to_string();
    let file_path = path.unwrap_or_else(|| Path::new("feed.xml"));
    fs::write(file_path, &rss_content).expect("Failed to write RSS feed to file");
}
