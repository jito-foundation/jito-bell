use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap; // Use BTreeMap for consistent ordering
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Deserialize)]
struct Tweet {
    id: String,
    text: String,
    author_id: Option<String>,
    created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct TweetResponse {
    data: Option<Vec<Tweet>>,
    meta: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct TweetRequest {
    text: String,
}

pub struct TwitterBot {
    client: Client,
    bearer_token: String,
    api_key: String,
    api_secret: String,
    access_token: String,
    access_token_secret: String,
    last_mention_id: Option<String>,
}

impl TwitterBot {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv::dotenv().ok();

        let bearer_token = std::env::var("TWITTER_BEARER_TOKEN")?;
        let api_key = std::env::var("TWITTER_API_KEY")?;
        let api_secret = std::env::var("TWITTER_API_SECRET")?;
        let access_token = std::env::var("TWITTER_ACCESS_TOKEN")?;
        let access_token_secret = std::env::var("TWITTER_ACCESS_TOKEN_SECRET")?;

        // Debug: Print (partial) credentials to verify they're loaded
        println!(
            "🔑 API Key: {}...",
            &api_key[..std::cmp::min(8, api_key.len())]
        );
        println!(
            "🔑 Access Token: {}...",
            &access_token[..std::cmp::min(8, access_token.len())]
        );
        println!("🔑 Credentials loaded successfully");

        Ok(Self {
            client: Client::new(),
            bearer_token,
            api_key,
            api_secret,
            access_token,
            access_token_secret,
            last_mention_id: None,
        })
    }

    // Generate OAuth 1.0a signature - Based on working JavaScript implementation
    fn generate_oauth_header(&self, method: &str, url: &str) -> String {
        use hmac::{Hmac, Mac};
        use percent_encoding::{utf8_percent_encode, AsciiSet, CONTROLS};
        use sha1::Sha1;

        // OAuth percent encoding (RFC 3986)
        const OAUTH_ENCODE_SET: &AsciiSet = &CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'#')
            .add(b'%')
            .add(b'&')
            .add(b'+')
            .add(b',')
            .add(b'/')
            .add(b':')
            .add(b';')
            .add(b'<')
            .add(b'=')
            .add(b'>')
            .add(b'?')
            .add(b'@')
            .add(b'[')
            .add(b'\\')
            .add(b']')
            .add(b'^')
            .add(b'`')
            .add(b'{')
            .add(b'|')
            .add(b'}')
            .add(b'~')
            .add(b'!')
            .add(b'*')
            .add(b'\'')
            .add(b'(')
            .add(b')');

        type HmacSha1 = Hmac<Sha1>;

        let timestamp = chrono::Utc::now().timestamp().to_string();
        let nonce = format!("{:x}{:x}", rand::random::<u64>(), rand::random::<u64>());

        // Build OAuth parameters exactly like the JavaScript sample
        let mut oauth_params = BTreeMap::new();
        oauth_params.insert("oauth_consumer_key", self.api_key.as_str());
        oauth_params.insert("oauth_nonce", nonce.as_str());
        oauth_params.insert("oauth_signature_method", "HMAC-SHA1");
        oauth_params.insert("oauth_timestamp", timestamp.as_str());
        oauth_params.insert("oauth_token", self.access_token.as_str());
        oauth_params.insert("oauth_version", "1.0");

        // Create parameter string (only OAuth params, no body params for JSON POST)
        let param_string = oauth_params
            .iter()
            .map(|(k, v)| {
                format!(
                    "{}={}",
                    utf8_percent_encode(k, OAUTH_ENCODE_SET),
                    utf8_percent_encode(v, OAUTH_ENCODE_SET)
                )
            })
            .collect::<Vec<_>>()
            .join("&");

        // Create signature base string exactly like JavaScript sample
        let base_string = format!(
            "{}&{}&{}",
            method.to_uppercase(),
            utf8_percent_encode(url, OAUTH_ENCODE_SET),
            utf8_percent_encode(&param_string, OAUTH_ENCODE_SET)
        );

        println!("🔍 Debug - Base string: {}", base_string);

        // Create signing key
        let signing_key = format!(
            "{}&{}",
            utf8_percent_encode(&self.api_secret, OAUTH_ENCODE_SET),
            utf8_percent_encode(&self.access_token_secret, OAUTH_ENCODE_SET)
        );

        // Generate signature using HMAC-SHA1
        let mut mac = HmacSha1::new_from_slice(signing_key.as_bytes()).unwrap();
        mac.update(base_string.as_bytes());
        let signature = base64::encode(mac.finalize().into_bytes());

        // Build authorization header like JavaScript sample
        let auth_header = format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_nonce=\"{}\", oauth_signature=\"{}\", oauth_signature_method=\"HMAC-SHA1\", oauth_timestamp=\"{}\", oauth_token=\"{}\", oauth_version=\"1.0\"",
            utf8_percent_encode(&self.api_key, OAUTH_ENCODE_SET),
            utf8_percent_encode(&nonce, OAUTH_ENCODE_SET),
            utf8_percent_encode(&signature, OAUTH_ENCODE_SET),
            utf8_percent_encode(&timestamp, OAUTH_ENCODE_SET),
            utf8_percent_encode(&self.access_token, OAUTH_ENCODE_SET)
        );

        println!("🔍 Debug - OAuth header: {}", auth_header);
        auth_header
    }

    // Post a tweet using OAuth 1.0a - matching JavaScript sample pattern
    pub async fn tweet(&self, text: &str) -> Result<(), Box<dyn std::error::Error>> {
        println!("🚀 Attempting to post tweet: {}", text);

        let url = "https://api.twitter.com/2/tweets";
        let tweet_data = TweetRequest {
            text: text.to_string(),
        };

        let body = serde_json::to_string(&tweet_data)?;
        let auth_header = self.generate_oauth_header("POST", url);

        let response = self
            .client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
            .header("User-Agent", "TwitterBotRust/1.0")
            .body(body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if status.is_success() {
            println!("✅ Tweet posted successfully!");
            println!("📝 Response: {}", response_text);
        } else {
            println!("❌ Failed to post tweet. Status: {}", status);
            println!("📝 Response: {}", response_text);

            // More detailed error analysis
            if status == 401 {
                println!("💡 This is still a 401 error. Please ensure you've:");
                println!("   1. Set app permissions to 'Read and Write' in Developer Portal");
                println!("   2. REGENERATED your access tokens AFTER changing permissions");
                println!("   3. Updated your .env file with the NEW tokens");
            }

            return Err(format!("Tweet failed with status: {}", status).into());
        }

        Ok(())
    }

    // Get mentions
    pub async fn get_mentions(&mut self) -> Result<Vec<Tweet>, Box<dyn std::error::Error>> {
        let mut url =
            "https://api.twitter.com/2/tweets/search/recent?query=@YOUR_USERNAME".to_string();

        if let Some(ref last_id) = self.last_mention_id {
            url.push_str(&format!("&since_id={}", last_id));
        }

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        let tweet_response: TweetResponse = response.json().await?;

        if let Some(tweets) = tweet_response.data {
            if let Some(latest_tweet) = tweets.first() {
                self.last_mention_id = Some(latest_tweet.id.clone());
            }
            Ok(tweets)
        } else {
            Ok(vec![])
        }
    }

    // Reply to a tweet
    pub async fn reply_to_tweet(
        &mut self,
        tweet_id: &str,
        reply_text: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let url = "https://api.twitter.com/2/tweets";

        let tweet_data = serde_json::json!({
            "text": reply_text,
            "reply": {
                "in_reply_to_tweet_id": tweet_id
            }
        });

        let body = serde_json::to_string(&tweet_data)?;
        // let params = HashMap::new();
        let auth_header = self.generate_oauth_header("POST", url);

        let response = self
            .client
            .post(url)
            .header("Authorization", auth_header)
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await?;

        if response.status().is_success() {
            println!("✅ Reply sent successfully to tweet {}", tweet_id);
        } else {
            let error_text = response.text().await?;
            println!("❌ Failed to send reply: {}", error_text);
        }

        Ok(())
    }

    // Bot content arrays
    fn get_quotes(&self) -> &[&str] {
        &[
            "The only way to do great work is to love what you do. 💼",
            "Innovation distinguishes between a leader and a follower. 🚀",
            "Life is what happens while you're busy making other plans. 🌟",
            "The future belongs to those who believe in their dreams. ✨",
            "Focus on the light during your darkest moments. 🕯️",
        ]
    }

    fn get_rust_tips(&self) -> &[&str] {
        &[
            "🦀 Rust tip: Use `cargo clippy` to catch common mistakes and improve your code!",
            "⚡ Remember: Rust's ownership system prevents memory leaks at compile time!",
            "🔧 Pro tip: Use `Result<T, E>` for error handling - it's safer than exceptions!",
            "🎯 Rust's pattern matching with `match` is incredibly powerful - use it!",
            "📦 Don't forget to run `cargo fmt` to keep your code beautifully formatted!",
        ]
    }

    // Post random content
    pub async fn post_random_quote(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let quotes = self.get_quotes();
        let quote = quotes[rand::random::<usize>() % quotes.len()];
        self.tweet(quote).await
    }

    pub async fn post_random_tip(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let tips = self.get_rust_tips();
        let tip = tips[rand::random::<usize>() % tips.len()];
        self.tweet(tip).await
    }

    // Process mentions and reply
    pub async fn process_mentions(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mentions = self.get_mentions().await?;

        for mention in mentions {
            println!("📩 Processing mention: {}", mention.text);

            let reply = if mention.text.to_lowercase().contains("hello") {
                "Hello! 👋 Thanks for reaching out! How can I help you today?"
            } else if mention.text.to_lowercase().contains("rust") {
                "🦀 Rust is amazing! It's fast, safe, and has zero-cost abstractions. What would you like to know about Rust?"
            } else if mention.text.to_lowercase().contains("help") {
                "🤝 I'm here to help! I can share programming tips, Rust advice, or just chat. What do you need?"
            } else {
                "Thanks for the mention! 🙏 I appreciate you reaching out. Have a great day!"
            };

            self.reply_to_tweet(&mention.id, reply).await?;

            // Small delay to avoid rate limiting
            sleep(Duration::from_secs(2)).await;
        }

        Ok(())
    }

    // Main bot loop
    pub async fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🤖 Twitter Bot starting up...");

        // Post initial greeting
        self.tweet("🦀 Rust Twitter Bot is now online! Ready to share tips, quotes, and engage with the community! #RustLang #TwitterBot").await?;

        let mut quote_counter = 0;
        let mut tip_counter = 0;

        loop {
            // Process mentions every iteration
            if let Err(e) = self.process_mentions().await {
                eprintln!("Error processing mentions: {}", e);
            }

            // Post content every 10 iterations (adjust timing as needed)
            if quote_counter >= 20 {
                // Post quote every ~20 minutes
                if let Err(e) = self.post_random_quote().await {
                    eprintln!("Error posting quote: {}", e);
                }
                quote_counter = 0;
            }

            if tip_counter >= 30 {
                // Post tip every ~30 minutes
                if let Err(e) = self.post_random_tip().await {
                    eprintln!("Error posting tip: {}", e);
                }
                tip_counter = 0;
            }

            quote_counter += 1;
            tip_counter += 1;

            // Wait 1 minute before next iteration
            sleep(Duration::from_secs(60)).await;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut bot = TwitterBot::new()?;
    bot.run().await?;
    Ok(())
}
