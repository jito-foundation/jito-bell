# Jito Bell

![Jito Bell](./docs/assets/images/jito-bell-logo-full.png)

## Overview

- Track live Solana transactions
- Send notification to several destination (Slack, Telegram, Discord)

## Programs

### [Stake Pool](https://github.com/solana-program/stake-pool/blob/main/program/src/lib.rs)

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

## Getting Started

### Build

```bash
docker build -t jito-bell .
```

### Run

```bash
docker run jito-bell --confilg-file /etc/jito-bell/jito_bell_config.yaml
```

## References
- https://github.com/rpcpool/yellowstone-grpc/blob/master/examples/rust/src/bin/tx-blocktime.rs
