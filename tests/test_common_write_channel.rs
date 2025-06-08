mod common;

use common::{MockFileSystem, TempEnv, acquire_env_lock};
use feed_crafter::common::{create_feed, write_channel};

#[test]
fn test_write_channel_with_default_path() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_LINK", "https://example.com");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    let mock_fs = MockFileSystem::new();

    // Create a dummy channel
    let channel = create_feed();

    write_channel(&channel, None, &mock_fs);

    // Verify the file was written to the default path
    assert!(mock_fs.was_file_written("./feed/feed.xml"));
    let written_content = mock_fs.get_written_content("./feed/feed.xml").unwrap();
    assert!(written_content.contains("<?xml"));
    assert!(written_content.contains("<rss"));
}

#[test]
fn test_write_channel_with_custom_path() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_LINK", "https://example.com");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    let mock_fs = MockFileSystem::new();

    // Create a dummy channel
    let channel = create_feed();

    write_channel(&channel, Some("./custom/path.xml"), &mock_fs);

    // Verify the file was written to the custom path
    assert!(mock_fs.was_file_written("./custom/path.xml"));
    let written_content = mock_fs.get_written_content("./custom/path.xml").unwrap();
    assert!(written_content.contains("<?xml"));
    assert!(written_content.contains("<rss"));
}
