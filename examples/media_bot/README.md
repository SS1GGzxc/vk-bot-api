# Media Bot Example

This example demonstrates the media upload and download features of vk-bot-api.

## Features

### Upload Features
- **Send Photos** - Upload and send photo files
- **Send Documents** - Upload and send any document type
- **Send Voice Messages** - Upload and send voice messages (OGG Opus format)

### Download Features
- **Download Photos** - Save received photos (highest resolution)
- **Download Documents** - Save received documents
- **Download Audio** - Save audio files
- **Download Voice Messages** - Save voice messages as MP3
- **Download Video Thumbnails** - Save video previews
- **Download Stickers** - Save sticker images
- **Forward Attachments** - Forward attachments to other users

## Setup

1. Copy the environment file:
```bash
cp .env.example .env
```

2. Edit `.env` and add your VK token and group ID:
```
VK_TOKEN=your_token_here
VK_GROUP_ID=123456789
```

3. Run the bot:
```bash
cargo run
```

## Usage

### Bot Commands

Send these commands to the bot:

- `/help` - Show available commands
- `/photo` - Bot sends an example photo
- `/doc` - Bot sends an example document
- `/voice` - Voice message information
- `/stats` - Show bot statistics

### Attachment Handling

The bot automatically handles incoming attachments:

1. **Send a photo** to the bot → It downloads and forwards it back
2. **Send a document** → It downloads and reports file size
3. **Send audio** → It downloads the audio file
4. **Send voice message** → It downloads as MP3
5. **Send video** → It downloads the thumbnail
6. **Send sticker** → It downloads the sticker image

## Code Examples

### Sending a Photo

```rust
// Read photo from file
let photo_data = std::fs::read("photo.jpg")?;

// Send to user
api.send_photo(
    peer_id,
    photo_data,
    "photo.jpg",
    Some("Check out this photo!")
).await?;
```

### Sending a Document

```rust
// Read document
let doc_data = std::fs::read("document.pdf")?;

// Send with custom title
api.send_document(
    peer_id,
    doc_data,
    "document.pdf",
    Some("Important Document"),
    Some("Here is the file")
).await?;
```

### Sending a Voice Message

```rust
// Voice must be in OGG Opus format
let voice_data = std::fs::read("voice.ogg")?;

api.send_voice_message(
    peer_id,
    voice_data,
    "voice.ogg"
).await?;
```

### Downloading a Photo

```rust
// In message handler
if let Some(photo) = message.attachments.first()
    .and_then(|a| a.photo.as_ref()) {
    
    // Download the photo (highest resolution)
    let downloaded = api.download_photo(photo).await?;
    
    // Save to disk
    std::fs::write("downloaded.jpg", &downloaded.data)?;
    
    println!("Downloaded {} bytes", downloaded.data.len());
}
```

### Downloading Any File

```rust
// Download from URL
let file = api.download_file("https://example.com/file.jpg").await?;

// Access data and content type
println!("Size: {} bytes", file.data.len());
println!("Type: {:?}", file.content_type);
```

### Forwarding Attachments

```rust
// Forward an existing attachment
let attachment = "photo12345_67890";

api.forward_attachment(
    another_peer_id,
    attachment,
    Some("Forwarded from another chat")
).await?;
```

## API Reference

### Upload Methods

- `photos_get_messages_upload_server(peer_id)` - Get upload URL for photos
- `photos_save_messages_photo(photo, server, hash)` - Save uploaded photo
- `docs_get_messages_upload_server(peer_id, doc_type)` - Get upload URL for documents
- `docs_save(file, title, tags)` - Save uploaded document
- `upload_file(upload_url, file_data, filename)` - Raw file upload

### High-Level Send Methods

- `send_photo(peer_id, photo_data, filename, caption)` - Upload and send photo
- `send_document(peer_id, file_data, filename, title, caption)` - Upload and send document
- `send_voice_message(peer_id, audio_data, filename)` - Upload and send voice

### Download Methods

- `download_file(url)` - Download any file
- `download_photo(photo)` - Download photo (best quality)
- `download_document(doc)` - Download document
- `download_audio(audio)` - Download audio file
- `download_video_thumbnail(video)` - Download video thumbnail
- `download_sticker(sticker)` - Download sticker
- `download_audio_message(audio_msg)` - Download voice message (MP3)
- `forward_attachment(peer_id, attachment, caption)` - Forward attachment

## Notes

- Voice messages must be in OGG Opus format
- Photos are downloaded at the highest available resolution
- Audio downloads may fail for restricted/copyrighted content
- Large files may take time to upload/download
