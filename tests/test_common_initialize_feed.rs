mod common;

use common::{MockFileSystem, TempEnv, acquire_env_lock};
use feed_crafter::common::initialize_feed;

#[test]
fn test_initialize_feed_when_file_exists() {
    let mock_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
    <channel>
        <title>Test Feed</title>
        <description>A test RSS feed</description>
        <link>https://example.com</link>
    </channel>
</rss>"#;

    let mock_fs = MockFileSystem::with_existing_file(mock_xml.to_string());

    let channel = initialize_feed(&mock_fs);

    // Assert the channel was loaded from the mock XML
    assert_eq!(channel.title(), "Test Feed");
    assert_eq!(channel.description(), "A test RSS feed");

    // Verify no file was written (since we loaded from existing)
    assert!(!mock_fs.was_file_written("./feed/feed.xml"));
}

#[test]
fn test_initialize_feed_when_file_does_not_exist() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_LINK", "https://example.com");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    let mock_fs = MockFileSystem::new();

    let channel = initialize_feed(&mock_fs);

    assert_eq!(channel.title(), "Test Channel");
    assert_eq!(channel.link(), "https://example.com");
    assert_eq!(channel.description(), "Test channel description");

    // Verify that the new feed was written to file
    assert!(mock_fs.was_file_written("./feed/feed.xml"));
    let written_content = mock_fs.get_written_content("./feed/feed.xml").unwrap();
    assert!(written_content.contains("<?xml"));
    assert!(written_content.contains("<rss"));
}

#[test]
fn test_initialize_feed_with_empty_filesystem() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_LINK", "https://example.com");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    // This simulates a completely empty filesystem
    let mock_fs = MockFileSystem::new();

    let channel = initialize_feed(&mock_fs);

    assert_eq!(channel.title(), "Test Channel");
    assert_eq!(channel.link(), "https://example.com");
    assert_eq!(channel.description(), "Test channel description");

    // Verify the new feed was written
    assert!(mock_fs.was_file_written("./feed/feed.xml"));
}
