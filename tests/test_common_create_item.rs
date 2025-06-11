use chrono::DateTime;
use feed_crafter::common::create_item;
use rss::Item;

fn verify_guid(item: Item) {
    // Verify GUID properties
    let guid = item.guid.unwrap();
    assert!(!guid.value.is_empty());
    assert!(!guid.permalink);

    // Verify GUID is a valid UUID
    assert!(uuid::Uuid::parse_str(&guid.value).is_ok());
}

fn verify_date(item: Item) {
    // Verify that pub_date is a valid RFC2822 date
    let date_string = item.pub_date.unwrap();
    assert!(DateTime::parse_from_rfc2822(&date_string).is_ok());
}

#[test]
fn test_create_item_success() {
    let title = "Test Title".to_string();
    let description = "Test Description".to_string();
    let link = "http://example.com".to_string();
    let item = create_item(title.clone(), Some(description.clone()), Some(link.clone()));

    assert_eq!(item.title, Some(title));
    assert_eq!(item.description, Some(description));
    assert_eq!(item.link, Some(link));
    assert!(item.guid.is_some());
    assert!(item.pub_date.is_some());
    verify_guid(item.clone());
    verify_date(item.clone());
}

#[test]
fn test_create_item_with_title_only() {
    let title = "Test Title".to_string();
    let item = create_item(title.clone(), None, None);

    assert_eq!(item.title, Some(title));
    assert!(item.description.is_none());
    assert!(item.link.is_none());
    assert!(item.guid.is_some());
    assert!(item.pub_date.is_some());
    verify_guid(item.clone());
    verify_date(item.clone());
}

#[test]
fn test_create_item_with_unicode() {
    let title = "测试标题 🚀".to_string();
    let description = "测试描述 🎉".to_string();
    let link = "http://example.com".to_string();
    let item = create_item(title.clone(), Some(description.clone()), Some(link.clone()));

    assert_eq!(item.title, Some(title));
    assert_eq!(item.description, Some(description));
    assert_eq!(item.link, Some(link));
    assert!(item.guid.is_some());
    assert!(item.pub_date.is_some());
    verify_guid(item.clone());
    verify_date(item.clone());
}

#[test]
fn test_create_item_guid_uniqueness() {
    let title = "Test Title".to_string();

    let item1 = create_item(title.clone(), None, None);
    let item2 = create_item(title.clone(), None, None);

    let guid1 = item1.guid.unwrap();
    let guid2 = item2.guid.unwrap();

    // Each item should have a unique GUID
    assert_ne!(guid1.value, guid2.value);

    // Both should be valid UUIDs
    assert!(uuid::Uuid::parse_str(&guid1.value).is_ok());
    assert!(uuid::Uuid::parse_str(&guid2.value).is_ok());
}
