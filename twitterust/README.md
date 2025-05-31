# Twitterust

Async Rust library for interacting with Twitter API v2 using OAuth 1.0a authentication.

## Features

- ✅ OAuth 1.0a authentication
- ✅ Tweet posting

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
twitterust = "0.0.1"
tokio = { version = "1.0", features = ["full"] }
```

## Quick Start

```rust
use twitterust::{TwitterClient, TwitterCredentials};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let credentials = TwitterCredentials::new(
        "your_api_key",
        "your_api_secret", 
        "your_access_token",
        "your_access_token_secret"
    );
    
    let client = TwitterClient::new(credentials);
    
    // Post a tweet
    let tweet = client.tweet("Hello from Rust! 🦀").await?;
    println!("Posted tweet: {}", tweet.id);
    
    // Reply to a tweet
    client.reply(&tweet.id, "This is a reply!").await?;
    
    // Search tweets
    let tweets = client.search_recent_tweets("#rust", Some(10)).await?;
    for tweet in tweets {
        println!("Found tweet: {}", tweet.text);
    }
    
    Ok(())
}
```

## Environment Variables

You can load credentials from environment variables:

```rust
let client = TwitterClient::from_env()?;
```

Expected variables:
- `TWITTER_API_KEY`
- `TWITTER_API_SECRET`
- `TWITTER_ACCESS_TOKEN`
- `TWITTER_ACCESS_TOKEN_SECRET`

## Error Handling

The library provides comprehensive error handling:

```rust
match client.tweet("Hello world").await {
    Ok(tweet) => println!("Success: {}", tweet.id),
    Err(TwitterError::Authentication(msg)) => eprintln!("Auth error: {}", msg),
    Err(TwitterError::RateLimit(msg)) => eprintln!("Rate limited: {}", msg),
    Err(TwitterError::BadRequest(msg)) => eprintln!("Bad request: {}", msg),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Advanced Usage

### Tweet with Poll

```rust
client.tweet_with_poll(
    "What's your favorite programming language?",
    vec!["Rust".to_string(), "Python".to_string(), "JavaScript".to_string()],
    60 // Duration in minutes
).await?;
```

### Custom Tweet Request

```rust
use twitter_api_rs::{TweetRequest, TweetReply};

let request = TweetRequest {
    text: "Custom tweet with reply".to_string(),
    reply: Some(TweetReply {
        in_reply_to_tweet_id: "123456789".to_string(),
        exclude_reply_user_ids: None,
    }),
    media: None,
    poll: None,
};

let tweet = client.create_tweet(request).await?;
```

## Rate Limiting

Be mindful of Twitter's rate limits. The library doesn't automatically handle rate limiting, so implement delays between requests:

```rust
use tokio::time::{sleep, Duration};

for i in 0..10 {
    client.tweet(&format!("Tweet number {}", i)).await?;
    sleep(Duration::from_secs(5)).await; // Wait 5 seconds between tweets
}
```

## License

- Apache License, Version 2.0

