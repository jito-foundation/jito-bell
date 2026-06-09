// Integration test for mainnet transaction:
// https://solscan.io/tx/Fkjru2spekPeHzAbb4ZptZssWUcdH5TddCXwzyejbQfhmf25WCBbwtutJwB95nSE2m3nifSQGQHLcQHshg6W9jf
//
// Squads V3 CreateTransaction on SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu.
// Data verified against on-chain RPC response.

use jito_bell::{
    ix_parser::{squads_v3::SquadsV3Program, InstructionParser},
    tx_parser::JitoTransactionParser,
    SquadsContext,
};
use solana_pubkey::Pubkey;
use solana_sdk::signature::Signature;
use std::str::FromStr;
use yellowstone_grpc_proto::geyser::{SubscribeUpdateTransaction, SubscribeUpdateTransactionInfo};
use yellowstone_grpc_proto::prelude::{
    CompiledInstruction, Message, Transaction, TransactionStatusMeta,
};

const TX_SIG: &str =
    "Fkjru2spekPeHzAbb4ZptZssWUcdH5TddCXwzyejbQfhmf25WCBbwtutJwB95nSE2m3nifSQGQHLcQHshg6W9jf";
const EXPECTED_MULTISIG: &str = "6f9mMaBZ1CxL5Aad1u2GDKXvkYmN9NAt3BBWvb2Dwzjt";
const EXPECTED_TRANSACTION: &str = "4DxJoJRS9dsorFxCoPbVtcpG64iyj2bjHrXgoW2AFqUY";
const EXPECTED_SQUADS_URL: &str =
    "https://v3.squads.so/transactions/NmY5bU1hQloxQ3hMNUFhZDF1MkdES1h2a1ltTjlOQXQzQkJXdmIyRHd6anQ=/tx/4DxJoJRS9dsorFxCoPbVtcpG64iyj2bjHrXgoW2AFqUY";

fn pubkey_bytes(s: &str) -> Vec<u8> {
    Pubkey::from_str(s).unwrap().to_bytes().to_vec()
}

fn make_update() -> SubscribeUpdateTransaction {
    // Account layout from on-chain RPC response:
    //   0: 9X5Zn4dFBJVqzaCBYeqmEUasbX3G1QFwbmj4Ht3zDeaC  (payer)
    //   1: 4DxJoJRS9dsorFxCoPbVtcpG64iyj2bjHrXgoW2AFqUY  (transaction account)
    //   2: 6f9mMaBZ1CxL5Aad1u2GDKXvkYmN9NAt3BBWvb2Dwzjt  (multisig)
    //   3: BsU2ejgC4oM6vPx4BkJFQEnpD165hDwGVjSTXtYrjtFy
    //   4: DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL
    //   5: FNpFVWAHuMtfR8YM7BVx3PquBHTMXRTekng71v2Jx2nu
    //   6: 11111111111111111111111111111111  (system program)
    //   7: ComputeBudget111111111111111111111111111111
    //   8: SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu  (Squads V3)
    let account_keys = vec![
        pubkey_bytes("9X5Zn4dFBJVqzaCBYeqmEUasbX3G1QFwbmj4Ht3zDeaC"),
        pubkey_bytes("4DxJoJRS9dsorFxCoPbVtcpG64iyj2bjHrXgoW2AFqUY"),
        pubkey_bytes("6f9mMaBZ1CxL5Aad1u2GDKXvkYmN9NAt3BBWvb2Dwzjt"),
        pubkey_bytes("BsU2ejgC4oM6vPx4BkJFQEnpD165hDwGVjSTXtYrjtFy"),
        pubkey_bytes("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL"),
        pubkey_bytes("FNpFVWAHuMtfR8YM7BVx3PquBHTMXRTekng71v2Jx2nu"),
        pubkey_bytes("11111111111111111111111111111111"),
        pubkey_bytes("ComputeBudget111111111111111111111111111111"),
        pubkey_bytes("SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu"),
    ];

    // CreateTransaction instruction:
    //   programIdIndex=8 (Squads V3), accounts=[2,1,0,6]
    //   data: anchor discriminator for "create_transaction" + authority_index arg
    let create_tx_ix = CompiledInstruction {
        program_id_index: 8,
        accounts: vec![2, 1, 0, 6],
        data: hex::decode("e3c135ef377e706901000000").unwrap(),
    };

    let sig_bytes = Signature::from_str(TX_SIG).unwrap().as_ref().to_vec();

    SubscribeUpdateTransaction {
        transaction: Some(SubscribeUpdateTransactionInfo {
            signature: sig_bytes.clone(),
            transaction: Some(Transaction {
                signatures: vec![sig_bytes],
                message: Some(Message {
                    account_keys,
                    instructions: vec![create_tx_ix],
                    ..Default::default()
                }),
            }),
            meta: Some(TransactionStatusMeta {
                err: None,
                log_messages: vec![
                    "Program SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu invoke [1]".to_string(),
                    "Program log: Instruction: CreateTransaction".to_string(),
                    "Program SMPLecH534NA9acpos4G6x7uf3LWbCAwZQE9e8ZekMu success".to_string(),
                ],
                ..Default::default()
            }),
            ..Default::default()
        }),
        slot: 423663996,
    }
}

#[test]
fn squads_v3_create_transaction_fires_and_produces_slack_message() {
    let parser = JitoTransactionParser::new(make_update());

    // Parser should have extracted exactly one instruction: CreateTransaction
    let create_tx = parser.instructions.iter().find_map(|ix| match ix {
        InstructionParser::SquadsV3(SquadsV3Program::CreateTransaction {
            multisig,
            transaction,
        }) => Some((*multisig, *transaction)),
        _ => None,
    });

    let (multisig, transaction) =
        create_tx.expect("expected a SquadsV3 CreateTransaction instruction");

    assert_eq!(multisig.to_string(), EXPECTED_MULTISIG);
    assert_eq!(transaction.to_string(), EXPECTED_TRANSACTION);
    assert_eq!(parser.transaction_signature, TX_SIG);

    // Build the Slack payload that would be posted to the webhook
    let ctx = SquadsContext::V3Transaction {
        multisig,
        transaction,
    };
    let payload = ctx.build_slack_payload(
        "Squads v3 transaction created",
        &parser.transaction_signature,
        "https://explorer.solana.com",
    );

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    // Header block
    assert_eq!(
        payload["blocks"][0]["text"]["text"],
        "Squads Transaction Created"
    );

    // Description block
    assert!(payload["blocks"][1]["text"]["text"]
        .as_str()
        .unwrap()
        .contains("Squads v3 transaction created"));

    let squads_field = &payload["blocks"][2]["fields"][0];
    assert!(squads_field["text"]
        .as_str()
        .unwrap()
        .contains(EXPECTED_SQUADS_URL));

    // Fields block: multisig, transaction, and tx signature all appear
    let fields = payload["blocks"][2]["fields"].to_string();
    assert!(fields.contains(EXPECTED_MULTISIG));
    assert!(fields.contains(EXPECTED_TRANSACTION));
    assert!(fields.contains(TX_SIG));
}
