use vk_bot_api::utils::{create_mention, create_hashtag, escape_text, truncate_text, split_message};

#[test]
fn test_create_mention() {
    assert_eq!(create_mention(123, "Ivan"), "[id123|Ivan]");
}

#[test]
fn test_create_hashtag() {
    assert_eq!(create_hashtag("hello world"), "#hello_world");
    assert_eq!(create_hashtag("test"), "#test");
}

#[test]
fn test_escape_text() {
    assert_eq!(escape_text("hello <world> & others"), "hello &lt;world&gt; &amp; others");
}

#[test]
fn test_truncate_text() {
    assert_eq!(truncate_text("hello world", 100), "hello world");
    assert_eq!(truncate_text("hello world", 8), "hello...");
}

#[test]
fn test_split_message() {
    let msg = "123456789012345";
    let chunks = split_message(msg, 10);
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0], "1234567890");
    assert_eq!(chunks[1], "12345");
}
