---
title: Telegram
category: Jekyll
layout: post
weight: 1
---

## Step 1: Create a Telegram Bot

1. Open the Telegram app and search for "@BotFather"
2. Start a chat with BotFather and send the command `/newbot`
3. Follow the prompts to name your bot (e.g., "Jito Bell Bot")
4. Choose a username for your bot (must end with "bot", e.g., "JitoBellBot")
5. Once created, BotFather will provide a bot token - save this securely

## Step 2: Create a Telegram Channel or Group

1. Create a new channel or group in Telegram where notifications will be sent
2. Make your bot an administrator of the channel/group
    - For channels: Got to channel info -> Administrators -> Add Administrator -> search for your bot
    - For groups: Go to group info -> Administrators -> Add Admin -> search for your bot
3. Give the bot permission to post messages

## Step 3: Get Chat ID
### For a channel:

1. Post any message to the channel
2. Visit https://api.telegram.org/bot<YOUR_BOT_TOKEN>/getUpdates in your browser (replace <YOUR_BOT_TOKEN> with your actual token)
3. Look for the "chat" object and note the "id" value (usually negative for channels, e.g., "-1001234567890")

### For a group:

1. Add the bot to the group
2. Send a message in the group
3. Visit the same URL as above and find the "chat" object with the "id" value

## References
- https://core.telegram.org/api
