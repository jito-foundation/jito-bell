//! # Twitter API Rust Library
//!
//! A modern, async Rust library for interacting with Twitter API v2.
//!
//! ## Features
//!
//! - OAuth 1.0a authentication
//! - Tweet posting and management
//! - User mentions and replies
//! - Rate limiting friendly
//! - Async/await support
//! - Comprehensive error handling
//!
//! ## Quick Start
//!
//! ```rust
//! use twitterust::{TwitterClient, TwitterCredentials};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let credentials = TwitterCredentials {
//!         api_key: "your_api_key".to_string(),
//!         api_secret: "your_api_secret".to_string(),
//!         access_token: "your_access_token".to_string(),
//!         access_token_secret: "your_access_token_secret".to_string(),
//!     };
//!     
//!     let client = TwitterClient::new(credentials);
//!     client.tweet("Hello from Rust! 🦀").await?;
//!     
//!     Ok(())
//! }
//! ```

mod auth;
mod client;
mod error;
mod types;

pub use auth::TwitterCredentials;
pub use client::TwitterClient;
pub use error::TwitterError;
pub use types::*;
