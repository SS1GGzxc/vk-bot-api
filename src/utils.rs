use chrono::{DateTime, Utc};
use serde_json::json;

#[cfg(feature = "uuid")]
use uuid::Uuid;

/// Generate random ID for messages
pub fn generate_random_id() -> i64 {
    #[cfg(feature = "uuid")]
    {
        let uuid = Uuid::new_v4();
        (uuid.as_u128() & 0x7FFFFFFFFFFFFFFF) as i64
    }

    #[cfg(not(feature = "uuid"))]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }
}

/// Parse VK date string to DateTime
pub fn parse_vk_date(date: i64) -> DateTime<Utc> {
    DateTime::from_timestamp(date, 0).unwrap_or_else(Utc::now)
}

/// Format date for display
pub fn format_date(date: i64) -> String {
    let dt = parse_vk_date(date);
    dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Create mention string for user
pub fn create_mention(user_id: i64, text: &str) -> String {
    format!("[id{}|{}]", user_id, text)
}

/// Create hash tag
pub fn create_hashtag(tag: &str) -> String {
    format!("#{}", tag.replace(' ', "_"))
}

/// Parse payload from string
pub fn parse_payload(payload: &str) -> Option<serde_json::Value> {
    serde_json::from_str(payload).ok()
}

/// Create payload for button
pub fn create_payload(command: &str, data: Option<serde_json::Value>) -> serde_json::Value {
    let mut payload = json!({
        "command": command,
    });

    if let Some(data) = data
        && let Some(obj) = payload.as_object_mut()
    {
        for (key, value) in data.as_object().unwrap() {
            obj.insert(key.clone(), value.clone());
        }
    }

    payload
}

/// Escape special characters for VK
pub fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Truncate text for preview
pub fn truncate_text(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        text.to_string()
    } else {
        format!("{}...", &text[..max_len - 3])
    }
}

/// Split message into chunks (VK has 4096 char limit)
pub fn split_message(text: &str, max_chunk_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut remaining = text;

    while remaining.len() > max_chunk_size {
        let split_pos = remaining[..max_chunk_size]
            .rfind('\n')
            .unwrap_or(max_chunk_size);

        chunks.push(remaining[..split_pos].to_string());
        remaining = &remaining[split_pos..];

        if remaining.starts_with('\n') {
            remaining = &remaining[1..];
        }
    }

    if !remaining.is_empty() {
        chunks.push(remaining.to_string());
    }

    chunks
}
