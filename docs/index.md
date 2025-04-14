---
title: Jito Bell
category: Jekyll
layout: home
---

Jito Bell supports multiple notification channels to alert you when transactions of interest are detected on Solana. This guide covers how to set up each supported integration.

## Supported Platforms

- Telegram
- Slack
- Discord

## Configuration Overview

Add notification channels to your `jito_bell_config.yaml` file in the `notifications` section:

```yaml
notifications:
  slack:
    webhook_url: "https://hooks.slack.com/services/XXXXXXXXX/XXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXXXX"
    channel: "#jito-alerts"
  
  discord:
    webhook_url: "https://discord.com/api/webhooks/XXXXXXXXXXXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  
  telegram:
    bot_token: "0123456789:XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    chat_id: "-1001234567890"
```

You can customize message formats for each platform in the `message_templates` section:

```yaml
message_templates:
  default: "{{description}} - Amount: {{amount}} SOL - Tx: https://explorer.solana.com/tx/{{tx_hash}}"
  slack: "<!here> {{description}} - Amount: {{amount}} SOL - <https://explorer.solana.com/tx/{{tx_hash}}|View Transaction>"
  discord: "@here {{description}} - Amount: {{amount}} SOL - [View Transaction](https://explorer.solana.com/tx/{{tx_hash}})"
```

Available variables in templates:

- `{{description}}`: The notification description from your configuration
- `{{amount}}`: The transaction amount in SOL
- `{{tx_hash}}`: The transaction hash/signature
- `{{timestamp}}`: The time when the transaction was processed

## Specifying Notification Destinations

For each monitored instruction, specify which notification channels to use:

```yaml
programs:
  spl_stake_pool:
    # ... existing configuration ...
    instructions:
      deposit_stake:
        # ... 
        notification:
          description: "Large JitoSOL stake deposit detected"
          destinations: ["telegram", "slack", "discord"]  # Send to all platforms
      withdraw_stake:
        # ...
        notification:
          description: "Large JitoSOL stake withdrawal detected"
          destinations: ["slack"]  # Send only to Slack
```

## Getting Started

### Create `jito_bell_config.yaml`

Copy `jito_bell_config.sample.yaml` to `jito_bell_config.yaml`

```yaml
programs:
  spl_stake_pool:
    program_id: "SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy"
    instructions:
      deposit_stake:
        pool_mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
        threshold: 1000.0  # SOL
        notification:
          description: "Large JitoSOL stake deposit detected"
          destinations: ["telegram"]
      withdraw_stake:
        pool_mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
        threshold: 1000.0  # SOL
        notification:
          description: "Large JitoSOL stake withdrawal detected"
          destinations: ["telegram"]
      deposit_sol:
        pool_mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
        threshold: 1000.0  # SOL
        notification:
          description: "Large SOL deposit to JitoSOL detected"
          destinations: ["telegram"]
      withdraw_sol:
        pool_mint: "J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn"
        threshold: 1000.0  # SOL
        notification:
          description: "Large SOL withdrawal from JitoSOL detected"
          destinations: ["telegram"]
  
notifications:
  slack:
    webhook_url: "https://hooks.slack.com/services/XXXXXXXXX/XXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXXXX"
    channel: "#jito-alerts"
  
  discord:
    webhook_url: "https://discord.com/api/webhooks/XXXXXXXXXXXXXXXXXX/XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
  
  telegram:
    bot_token: "0123456789:XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
    chat_id: "-1001234567890"

message_templates:
  default: "{{description}} - Amount: {{amount}} SOL - Tx: https://explorer.solana.com/tx/{{tx_hash}}"
  slack: "<!here> {{description}} - Amount: {{amount}} SOL - <https://explorer.solana.com/tx/{{tx_hash}}|View Transaction>"
  discord: "@here {{description}} - Amount: {{amount}} SOL - [View Transaction](https://explorer.solana.com/tx/{{tx_hash}})"
```

### Build

```bash
docker build -t jito-bell .
```

### Run Jito Bell app

We need rpc url for running Jito Bell app.

```bash
docker run jito-bell \
  -e "https://your-endpoint.com" \
  -x-token "your-token-here" \
  -account-include SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy \
  -config-file /etc/jito-bell/jito_bell_config.yaml
```

## License

This project is licensed under the Business Source License 1.1 - see the [LICENSE.md](../LICENSE.md) file for details.
