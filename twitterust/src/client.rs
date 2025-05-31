use crate::{
    auth::{OAuthSigner, TwitterCredentials},
    error::{Result, TwitterError},
    types::*,
};
use reqwest::Client;

/// Main Twitter API client
#[derive(Debug)]
pub struct TwitterClient {
    credentials: TwitterCredentials,
    client: Client,
    base_url: String,
}

impl TwitterClient {
    /// Create a new Twitter client with credentials
    pub fn new(credentials: TwitterCredentials) -> Self {
        Self {
            credentials,
            client: Client::new(),
            base_url: "https://api.twitter.com/2".to_string(),
        }
    }

    /// Create a client from environment variables
    pub fn from_env() -> Result<Self> {
        let credentials = TwitterCredentials::from_env().map_err(|e| {
            TwitterError::Authentication(format!(
                "Failed to load credentials from environment: {}",
                e
            ))
        })?;
        Ok(Self::new(credentials))
    }

    /// Post a simple text tweet
    pub async fn tweet(&self, text: impl Into<String>) -> Result<Tweet> {
        let request = TweetRequest {
            text: text.into(),
            reply: None,
            media: None,
            poll: None,
        };
        self.create_tweet(request).await
    }

    /// Reply to a tweet
    pub async fn reply(
        &self,
        tweet_id: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Tweet> {
        let request = TweetRequest {
            text: text.into(),
            reply: Some(TweetReply {
                in_reply_to_tweet_id: tweet_id.into(),
                exclude_reply_user_ids: None,
            }),
            media: None,
            poll: None,
        };
        self.create_tweet(request).await
    }

    /// Create a tweet with a poll
    pub async fn tweet_with_poll(
        &self,
        text: impl Into<String>,
        poll_options: Vec<String>,
        duration_minutes: u32,
    ) -> Result<Tweet> {
        let request = TweetRequest {
            text: text.into(),
            reply: None,
            media: None,
            poll: Some(TweetPoll {
                options: poll_options,
                duration_minutes,
            }),
        };
        self.create_tweet(request).await
    }

    /// Create a tweet (generic method)
    pub async fn create_tweet(&self, request: TweetRequest) -> Result<Tweet> {
        let url = format!("{}/tweets", self.base_url);
        let signer = OAuthSigner::new(&self.credentials);
        let auth_header = signer.generate_auth_header("POST", &url);

        let response = self
            .client
            .post(&url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("User-Agent", "TwitterApiRs/0.1.0")
            .json(&request)
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let tweet_response: TweetResponse = response.json().await?;
            Ok(tweet_response.data)
        } else {
            let error_body = response.text().await.unwrap_or_default();
            Err(TwitterError::from_status_and_body(
                status.as_u16(),
                &error_body,
            ))
        }
    }

    /// Get mentions of the authenticated user
    pub async fn get_mentions(&self, since_id: Option<&str>) -> Result<Vec<Tweet>> {
        let mut url = format!("{}/users/me/mentions", self.base_url);

        if let Some(since) = since_id {
            url.push_str(&format!("?since_id={}", since));
        }

        let signer = OAuthSigner::new(&self.credentials);
        let auth_header = signer.generate_auth_header("GET", &url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", auth_header)
            .header("Accept", "application/json")
            .header("User-Agent", "TwitterApiRs/0.1.0")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            let search_response: TweetSearchResponse = response.json().await?;
            Ok(search_response.data.unwrap_or_default())
        } else {
            let error_body = response.text().await.unwrap_or_default();
            Err(TwitterError::from_status_and_body(
                status.as_u16(),
                &error_body,
            ))
        }
    }

    /// Get authenticated user information
    pub async fn get_me(&self) -> Result<User> {
        let url = format!("{}/users/me", self.base_url);
        let signer = OAuthSigner::new(&self.credentials);
        let auth_header = signer.generate_auth_header("GET", &url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", auth_header)
            .header("Accept", "application/json")
            .header("User-Agent", "TwitterApiRs/0.1.0")
            .send()
            .await?;

        let status = response.status();
        if status.is_success() {
            #[derive(serde::Deserialize)]
            struct UserResponse {
                data: User,
            }
            let user_response: UserResponse = response.json().await?;
            Ok(user_response.data)
        } else {
            let error_body = response.text().await.unwrap_or_default();
            Err(TwitterError::from_status_and_body(
                status.as_u16(),
                &error_body,
            ))
        }
    }
}
