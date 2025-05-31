use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request to create a new tweet
#[derive(Debug, Serialize)]
pub struct TweetRequest {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply: Option<TweetReply>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media: Option<TweetMedia>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll: Option<TweetPoll>,
}

#[derive(Debug, Serialize)]
pub struct TweetReply {
    pub in_reply_to_tweet_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_reply_user_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TweetMedia {
    pub media_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tagged_user_ids: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TweetPoll {
    pub options: Vec<String>,
    pub duration_minutes: u32,
}

/// Response from creating a tweet
#[derive(Debug, Deserialize)]
pub struct TweetResponse {
    pub data: Tweet,
}

/// Tweet object
#[derive(Debug, Deserialize)]
pub struct Tweet {
    pub id: String,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_reply_to_user_id: Option<String>,
}

/// Response from searching tweets
#[derive(Debug, Deserialize)]
pub struct TweetSearchResponse {
    pub data: Option<Vec<Tweet>>,
    pub meta: Option<SearchMeta>,
}

#[derive(Debug, Deserialize)]
pub struct SearchMeta {
    pub result_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub newest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oldest_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_token: Option<String>,
}

/// User information
#[derive(Debug, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_image_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub public_metrics: Option<UserMetrics>,
}

#[derive(Debug, Deserialize)]
pub struct UserMetrics {
    pub followers_count: u64,
    pub following_count: u64,
    pub tweet_count: u64,
    pub listed_count: u64,
}
