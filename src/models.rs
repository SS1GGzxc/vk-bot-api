use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Main update type from VK
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Update {
    /// Update type
    #[serde(rename = "type")]
    pub update_type: String,
    /// Update object
    pub object: Value,
    /// Group ID
    pub group_id: i64,
    /// Event ID (for callbacks)
    #[serde(default)]
    pub event_id: Option<String>,
}

/// Event types
#[derive(Debug, Clone)]
pub enum Event {
    /// New message
    MessageNew(Message),
    /// Message edited
    MessageEdit(Message),
    /// Message reply
    MessageReply(Message),
    /// Message allow
    MessageAllow(MessageAllow),
    /// Message deny
    MessageDeny(MessageDeny),
    /// Message typing state
    MessageTypingState(MessageTypingState),
    /// Message event (callback)
    MessageEvent(MessageEvent),
    /// Unknown event
    Unknown(String, Value),
}

impl Event {
    /// Convert update to event
    pub fn from_update(update: &Update) -> Self {
        match update.update_type.as_str() {
            "message_new" => {
                if let Ok(message) = serde_json::from_value(update.object["message"].clone()) {
                    Event::MessageNew(message)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            "message_edit" => {
                if let Ok(message) = serde_json::from_value(update.object.clone()) {
                    Event::MessageEdit(message)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            "message_reply" => {
                if let Ok(message) = serde_json::from_value(update.object.clone()) {
                    Event::MessageReply(message)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            "message_allow" => {
                if let Ok(message_allow) = serde_json::from_value(update.object.clone()) {
                    Event::MessageAllow(message_allow)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            "message_deny" => {
                if let Ok(message_deny) = serde_json::from_value(update.object.clone()) {
                    Event::MessageDeny(message_deny)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            "message_typing_state" => {
                if let Ok(typing_state) = serde_json::from_value(update.object.clone()) {
                    Event::MessageTypingState(typing_state)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            "message_event" => {
                if let Ok(message_event) = serde_json::from_value(update.object.clone()) {
                    Event::MessageEvent(message_event)
                } else {
                    Event::Unknown(update.update_type.clone(), update.object.clone())
                }
            }
            _ => Event::Unknown(update.update_type.clone(), update.object.clone()),
        }
    }
}

/// Message structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Message {
    /// Message ID
    pub id: i64,
    /// Sender ID
    pub from_id: i64,
    /// Message text
    pub text: String,
    /// Peer ID (user/chat ID)
    pub peer_id: i64,
    /// Conversation message ID
    #[serde(default)]
    pub conversation_message_id: Option<i64>,
    /// Date of sending (Unix timestamp)
    pub date: i64,
    /// Message attachments
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    /// Reply message (if any)
    #[serde(default)]
    pub reply_message: Option<Box<Message>>,
    /// Forwarded messages
    #[serde(default)]
    pub fwd_messages: Vec<Message>,
    /// Important flag
    #[serde(default)]
    pub important: bool,
    /// Random ID
    #[serde(default)]
    pub random_id: Option<i64>,
    /// Payload (for buttons)
    #[serde(default)]
    pub payload: Option<String>,
    /// Geo location
    #[serde(default)]
    pub geo: Option<Geo>,
    /// Action (for chat actions)
    #[serde(default)]
    pub action: Option<Action>,
}

/// Message action (for chat events)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Action {
    /// Action type
    #[serde(rename = "type")]
    pub action_type: String,
    /// Member ID (for chat_invite_user, chat_kick_user)
    #[serde(default)]
    pub member_id: Option<i64>,
    /// Text (for chat_title_update)
    #[serde(default)]
    pub text: Option<String>,
    /// Email (for chat_invite_user_by_link)
    #[serde(default)]
    pub email: Option<String>,
}

/// Geo location
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Geo {
    /// Place ID
    #[serde(default)]
    pub place: Option<Place>,
    /// Coordinates
    pub coordinates: Coordinates,
}

/// Place information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Place {
    /// Place ID
    pub id: i64,
    /// Title
    pub title: String,
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
    /// Created timestamp
    pub created: i64,
    /// Icon URL
    #[serde(default)]
    pub icon: Option<String>,
    /// Country
    #[serde(default)]
    pub country: Option<String>,
    /// City
    #[serde(default)]
    pub city: Option<String>,
}

/// Coordinates
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Coordinates {
    /// Latitude
    pub latitude: f64,
    /// Longitude
    pub longitude: f64,
}

/// Message allow event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageAllow {
    /// User ID
    pub user_id: i64,
    /// Key
    pub key: String,
}

/// Message deny event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageDeny {
    /// User ID
    pub user_id: i64,
}

/// Message typing state event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageTypingState {
    /// User ID
    pub user_id: i64,
    /// Peer ID
    pub peer_id: i64,
    /// State
    pub state: String,
}

/// Message callback event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MessageEvent {
    /// User ID
    pub user_id: i64,
    /// Peer ID
    pub peer_id: i64,
    /// Event ID
    pub event_id: String,
    /// Payload
    #[serde(default)]
    pub payload: Option<HashMap<String, Value>>,
    /// Conversation message ID
    #[serde(default)]
    pub conversation_message_id: Option<i64>,
}

/// Message attachment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Attachment {
    /// Attachment type
    #[serde(rename = "type")]
    pub attachment_type: String,
    /// Photo
    #[serde(default)]
    pub photo: Option<Photo>,
    /// Video
    #[serde(default)]
    pub video: Option<Video>,
    /// Audio
    #[serde(default)]
    pub audio: Option<Audio>,
    /// Document
    #[serde(default)]
    pub doc: Option<Document>,
    /// Link
    #[serde(default)]
    pub link: Option<Link>,
    /// Market item
    #[serde(default)]
    pub market: Option<Market>,
    /// Market album
    #[serde(default)]
    pub market_album: Option<MarketAlbum>,
    /// Wall post
    #[serde(default)]
    pub wall: Option<WallPost>,
    /// Wall reply
    #[serde(default)]
    pub wall_reply: Option<WallReply>,
    /// Sticker
    #[serde(default)]
    pub sticker: Option<Sticker>,
    /// Gift
    #[serde(default)]
    pub gift: Option<Gift>,
    /// Poll
    #[serde(default)]
    pub poll: Option<Poll>,
    /// Audio message
    #[serde(default)]
    pub audio_message: Option<AudioMessage>,
}

/// Photo attachment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Photo {
    /// Photo ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Access key
    #[serde(default)]
    pub access_key: Option<String>,
    /// Photo sizes
    pub sizes: Vec<PhotoSize>,
    /// Text
    #[serde(default)]
    pub text: Option<String>,
    /// Date
    pub date: i64,
    /// Width
    #[serde(default)]
    pub width: Option<i32>,
    /// Height
    #[serde(default)]
    pub height: Option<i32>,
}

/// Photo size
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PhotoSize {
    /// Size type
    #[serde(rename = "type")]
    pub size_type: String,
    /// URL
    pub url: String,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

/// Video attachment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Video {
    /// Video ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Title
    pub title: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Duration
    pub duration: i32,
    /// Images
    pub image: Vec<VideoImage>,
    /// First frame images
    #[serde(default)]
    pub first_frame: Vec<VideoImage>,
    /// Date
    pub date: i64,
    /// Views
    #[serde(default)]
    pub views: Option<i32>,
    /// Comments
    #[serde(default)]
    pub comments: Option<i32>,
    /// Access key
    #[serde(default)]
    pub access_key: Option<String>,
    /// Player URL
    #[serde(default)]
    pub player: Option<String>,
}

/// Video image
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoImage {
    /// URL
    pub url: String,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

/// Audio attachment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Audio {
    /// Audio ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Artist
    pub artist: String,
    /// Title
    pub title: String,
    /// Duration
    pub duration: i32,
    /// URL
    #[serde(default)]
    pub url: Option<String>,
    /// Date
    pub date: i64,
    /// Album ID
    #[serde(default)]
    pub album_id: Option<i64>,
    /// Genre ID
    #[serde(default)]
    pub genre_id: Option<i64>,
}

/// Document attachment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Document {
    /// Document ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Title
    pub title: String,
    /// Size
    pub size: i64,
    /// Extension
    pub ext: String,
    /// URL
    pub url: String,
    /// Date
    pub date: i64,
    /// Type
    #[serde(default)]
    pub r#type: Option<i32>,
    /// Preview
    #[serde(default)]
    pub preview: Option<DocumentPreview>,
}

/// Document preview
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentPreview {
    /// Photo
    #[serde(default)]
    pub photo: Option<DocumentPreviewPhoto>,
    /// Graffiti
    #[serde(default)]
    pub graffiti: Option<Graffiti>,
    /// Audio message
    #[serde(default)]
    pub audio_message: Option<AudioMessage>,
}

/// Document preview photo
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DocumentPreviewPhoto {
    /// Sizes
    pub sizes: Vec<PhotoSize>,
}

/// Graffiti
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Graffiti {
    /// Src
    pub src: String,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

/// Audio message
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AudioMessage {
    /// Duration
    pub duration: i32,
    /// Waveform
    pub waveform: Vec<i32>,
    /// Link OGG
    pub link_ogg: String,
    /// Link MP3
    pub link_mp3: String,
}

/// Link attachment
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Link {
    /// URL
    pub url: String,
    /// Title
    #[serde(default)]
    pub title: Option<String>,
    /// Caption
    #[serde(default)]
    pub caption: Option<String>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Photo
    #[serde(default)]
    pub photo: Option<Photo>,
}

/// Market item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Market {
    /// Item ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Price
    pub price: MarketPrice,
    /// Photos
    pub photos: Vec<Photo>,
    /// Date
    pub date: i64,
    /// Access key
    #[serde(default)]
    pub access_key: Option<String>,
}

/// Market price
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketPrice {
    /// Amount
    pub amount: String,
    /// Currency
    pub currency: Currency,
    /// Text
    pub text: String,
}

/// Currency
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Currency {
    /// Currency ID
    pub id: i64,
    /// Name
    pub name: String,
}

/// Market album
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketAlbum {
    /// Album ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Title
    pub title: String,
    /// Photo
    #[serde(default)]
    pub photo: Option<Photo>,
}

/// Wall post
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WallPost {
    /// Post ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// From ID
    pub from_id: i64,
    /// Date
    pub date: i64,
    /// Text
    pub text: String,
    /// Attachments
    #[serde(default)]
    pub attachments: Vec<Attachment>,
}

/// Wall reply
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WallReply {
    /// Reply ID
    pub id: i64,
    /// From ID
    pub from_id: i64,
    /// Date
    pub date: i64,
    /// Text
    pub text: String,
}

/// Sticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Sticker {
    /// Product ID
    pub product_id: i64,
    /// Sticker ID
    pub sticker_id: i64,
    /// Images
    pub images: Vec<StickerImage>,
    /// Images with background
    #[serde(default)]
    pub images_with_background: Vec<StickerImage>,
}

/// Sticker image
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StickerImage {
    /// URL
    pub url: String,
    /// Width
    pub width: i32,
    /// Height
    pub height: i32,
}

/// Gift
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Gift {
    /// Gift ID
    pub id: i64,
    /// Thumbnail 256px
    pub thumb_256: String,
    /// Thumbnail 96px
    pub thumb_96: String,
    /// Thumbnail 48px
    pub thumb_48: String,
}

/// Poll
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Poll {
    /// Poll ID
    pub id: i64,
    /// Owner ID
    pub owner_id: i64,
    /// Created
    pub created: i64,
    /// Question
    pub question: String,
    /// Votes
    pub votes: i64,
    /// Answers
    pub answers: Vec<PollAnswer>,
    /// Anonymous
    pub anonymous: bool,
    /// Multiple
    #[serde(default)]
    pub multiple: Option<bool>,
    /// End date
    #[serde(default)]
    pub end_date: Option<i64>,
}

/// Poll answer
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PollAnswer {
    /// Answer ID
    pub id: i64,
    /// Text
    pub text: String,
    /// Votes
    pub votes: i64,
    /// Rate
    pub rate: f64,
}
