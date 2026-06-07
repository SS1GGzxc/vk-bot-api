use vk_bot_api::models::{Event, Update};
use serde_json::json;

#[test]
fn test_parse_message_new_update() {
    let update_json = json!({
        "type": "message_new",
        "object": {
            "message": {
                "id": 123,
                "date": 123456789,
                "peer_id": 456,
                "from_id": 789,
                "text": "Hello world!"
            }
        },
        "group_id": 1
    });

    let update: Update = serde_json::from_value(update_json).unwrap();
    let event = Event::from_update(&update);

    match event {
        Event::MessageNew(msg) => {
            assert_eq!(msg.id, 123);
            assert_eq!(msg.peer_id, 456);
            assert_eq!(msg.from_id, 789);
            assert_eq!(msg.text, "Hello world!");
        }
        _ => panic!("Expected MessageNew event"),
    }
}

#[test]
fn test_parse_message_reply_update() {
    let update_json = json!({
        "type": "message_reply",
        "object": {
            "id": 124,
            "date": 123456790,
            "peer_id": 456,
            "from_id": 789,
            "text": "Reply text"
        },
        "group_id": 1
    });

    let update: Update = serde_json::from_value(update_json).unwrap();
    let event = Event::from_update(&update);

    match event {
        Event::MessageReply(msg) => {
            assert_eq!(msg.id, 124);
            assert_eq!(msg.text, "Reply text");
        }
        _ => panic!("Expected MessageReply event"),
    }
}

#[test]
fn test_parse_unknown_event() {
    let update_json = json!({
        "type": "unknown_event_type",
        "object": {
            "some_field": "some_value"
        },
        "group_id": 1
    });

    let update: Update = serde_json::from_value(update_json).unwrap();
    let event = Event::from_update(&update);

    match event {
        Event::Unknown(event_type, object) => {
            assert_eq!(event_type, "unknown_event_type");
            assert_eq!(object["some_field"], "some_value");
        }
        _ => panic!("Expected Unknown event"),
    }
}
