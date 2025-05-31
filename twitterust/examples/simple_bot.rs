use twitterust::TwitterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Create client from environment variables
    let client = TwitterClient::from_env()?;

    // Simple tweet
    let tweet = client
        .tweet("🦀 Hello from the Twitterust library?")
        .await?;
    println!("Posted tweet: {}", tweet.id);

    Ok(())
}
