mod common;

use chrono::DateTime;
use common::*;
use feed_crafter::common::create_feed;

#[test]
fn test_create_feed_success() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_LINK", "https://example.com");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    let channel = create_feed();

    assert_eq!(channel.title(), "Test Channel");
    assert_eq!(channel.link(), "https://example.com");
    assert_eq!(channel.description(), "Test channel description");

    // Verify that last_build_date is set and is a valid RFC2822 date
    assert!(channel.last_build_date().is_some());
    let date_str = channel.last_build_date().unwrap();
    assert!(DateTime::parse_from_rfc2822(date_str).is_ok());
}

#[test]
#[should_panic(expected = "Expected a channel title in the environment")]
fn test_create_feed_missing_title() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_LINK", "https://example.com");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    create_feed();
}

#[test]
#[should_panic(expected = "Expected a channel link in the environment")]
fn test_create_feed_missing_link() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_DESCRIPTION", "Test channel description");

    create_feed();
}

#[test]
#[should_panic(expected = "Expected a channel description in the environment")]
fn test_create_feed_missing_description() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "Test Channel");
    temp_env.set("CHANNEL_LINK", "https://example.com");

    create_feed();
}

#[test]
fn test_create_feed_unicode_content() {
    let _guard = acquire_env_lock();
    let mut temp_env = TempEnv::new();

    temp_env.set("CHANNEL_TITLE", "测试频道 🚀");
    temp_env.set("CHANNEL_LINK", "https://example.com/测试");
    temp_env.set(
        "CHANNEL_DESCRIPTION",
        "This is a test with émojis 🎉 and ñoñ-ASCII characters",
    );

    let channel = create_feed();

    assert_eq!(channel.title(), "测试频道 🚀");
    assert_eq!(channel.link(), "https://example.com/测试");
    assert_eq!(
        channel.description(),
        "This is a test with émojis 🎉 and ñoñ-ASCII characters"
    );
}
