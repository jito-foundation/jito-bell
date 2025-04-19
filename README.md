# Jito Bell

![Jito Bell](./docs/assets/images/jito-bell-logo-full.png)

## Overview

- Track live Solana transactions
- Send notification to several destination (Slack, Telegram, Discord)

## Programs

### [SPL Stake Pool](https://github.com/solana-program/stake-pool/blob/main/program/src/lib.rs)

```bash
cargo r -- -e {endpoint} --x-token {x-token} --account-include SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy
```

- Program ID: SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy
- JitoSOL: J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn


#### Instructions

- [DepositStake](https://github.com/solana-program/stake-pool/blob/0740ef57b0cd202e948641545c2761557cc8c794/program/src/instruction.rs#L299)
- [WithdrawStake](https://github.com/solana-program/stake-pool/blob/0740ef57b0cd202e948641545c2761557cc8c794/program/src/instruction.rs#L337)
- [DepositSol](https://github.com/solana-program/stake-pool/blob/0740ef57b0cd202e948641545c2761557cc8c794/program/src/instruction.rs#L378)
- [WithdrawSol](https://github.com/solana-program/stake-pool/blob/0740ef57b0cd202e948641545c2761557cc8c794/program/src/instruction.rs#L405)

### [Jito Vault Program](https://github.com/jito-foundation/restaking)

```bash
cargo r -- -e {endpoint} --x-token {x-token} --account-include Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8
```

- Program ID: Vau1t6sLNxnzB7ZDsef8TLbPLfyZMYXH8WTNqUdm9g8


#### Instructions

- [MintTo](https://github.com/jito-foundation/restaking/blob/623b1816b9a93e3678c29c426e9b38ef2f324554/vault_sdk/src/instruction.rs#L132-L135)
- [EnqueueWithdrawal](https://github.com/jito-foundation/restaking/blob/623b1816b9a93e3678c29c426e9b38ef2f324554/vault_sdk/src/instruction.rs#L149-L151)

## Getting Started

### Create `.env` file

```bash
cp .env.example .env
```

### Create ` jito_bell_config.yaml`

```bash
cp jito_bell_config_example.yaml jito_bell_config.yaml
```

### Build

```bash
docker build -t jito-bell .
```

### Run

```bash
docker run jito-bell \
  -e "https://your-endpoint.com" \
  -x-token "your-token-here" \
  -account-include SPoo1Ku8WFXoNDMHPsrGSTSG1Y47rzgn41SLUNakuHy \
  -config-file /etc/jito-bell/jito_bell_config.yaml
```

## References
- https://github.com/rpcpool/yellowstone-grpc/blob/master/examples/rust/src/bin/tx-blocktime.rs
