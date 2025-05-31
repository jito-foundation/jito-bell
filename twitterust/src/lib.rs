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
//! ```

mod auth;
mod client;
mod error;
mod types;

pub use auth::TwitterCredentials;
pub use client::TwitterClient;
pub use error::TwitterError;
pub use types::*;
