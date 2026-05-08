//! # VK Bot API for Rust
//!
//! A modern, asynchronous, fully-featured VK Bot API library for Rust.
//!
//! ## Features
//!
//! - Full VK Bot API coverage
//! - Async/await support
//! - Strongly typed models
//! - Extensible handler system
//! - Multiple update delivery methods (Long Poll, Webhooks)
//! - Support for all attachment types
//! - Inline keyboards and callbacks
//! - Error handling and retries

// #![warn(missing_docs)]
#![allow(missing_docs)]
#![warn(rust_2018_idioms)]

// Все модули всегда доступны
pub mod api;
pub mod bot;
pub mod error;
pub mod handler;
pub mod keyboard;
pub mod models;
pub mod utils;

/// Prelude module for convenient imports
pub mod prelude {
    /// Re-export bot module types
    pub use crate::bot::{VkBot, VkBotBuilder};

    /// Re-export api module types
    pub use crate::api::{VkApi, VkApiBuilder};

    /// Re-export models
    pub use crate::models::*;

    /// Re-export keyboard
    pub use crate::keyboard::*;

    /// Re-export handler
    pub use crate::handler::*;

    /// Re-export error
    pub use crate::error::*;

    /// Re-export utils
    pub use crate::utils::*;
}

// Re-exports
pub use api::{
    AudioMessageInfo, DocInfo, DownloadedFile, LongPollServer, PhotoSizeInfo, SavedDocument,
    SavedPhoto, UploadResponse, UploadServer, VkApi, VkApiBuilder, VkApiConfig,
};
pub use bot::{VkBot, VkBotBuilder, VkBotConfig};
pub use error::{VkError, VkResult};
pub use handler::{DefaultMessageHandler, MessageHandler};

/// Logging macro for VK Bot API
#[macro_export]
macro_rules! vk_log {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        log::info!($($arg)*);
    };
}

/// Error logging macro for VK Bot API
#[macro_export]
macro_rules! vk_error {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        log::error!($($arg)*);
    };
}

/// Warning logging macro for VK Bot API
#[macro_export]
macro_rules! vk_warn {
    ($($arg:tt)*) => {
        #[cfg(feature = "logging")]
        log::warn!($($arg)*);
    };
}
