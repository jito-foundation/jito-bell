use twitterust::TwitterClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Create client from environment variables
    let client = TwitterClient::from_env()?;

    // Simple tweet
    let tweet = client
        .tweet("🦀 Hello from the Twitterust library!")
        .await?;
    println!("Posted tweet: {}", tweet.id);

    // Tweet with poll
    // sleep(Duration::from_secs(5)).await; // Rate limiting
    // let poll_tweet = client
    //     .tweet_with_poll(
    //         "What's your favorite Rust feature?",
    //         vec![
    //             "Memory Safety".to_string(),
    //             "Performance".to_string(),
    //             "Async/Await".to_string(),
    //             "Pattern Matching".to_string(),
    //         ],
    //         1, // 1 hour poll
    //     )
    //     .await?;
    // println!("Posted poll tweet: {}", poll_tweet.id);

    // Get mentions and reply to them
    // let mentions = client.get_mentions(None).await?;
    // for mention in mentions.iter().take(3) {
    //     // Limit to avoid rate limits
    //     println!("Processing mention: {}", mention.text);

    //     sleep(Duration::from_secs(2)).await; // Rate limiting

    //     let reply = client
    //         .reply(
    //             &mention.id,
    //             "Thanks for mentioning me! 🚀 This is an automated reply from my Rust bot.",
    //         )
    //         .await?;
    //     println!("Replied to mention: {}", reply.id);
    // }

    Ok(())
}
