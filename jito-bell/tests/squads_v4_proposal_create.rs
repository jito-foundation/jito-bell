// Integration test for mainnet transaction:
// https://solscan.io/tx/3RbCdn9YDPs5xGN4vMJBmraeozC2LoBxrySoEVraWFKwQpty1dLtv8yRXUzYJoUTSmLZKjqbjLXNZ2coMA18NMYx
//
// Squads V4 ProposalCreate on SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf.
// Data verified against on-chain RPC response.

use jito_bell::{
    ix_parser::{squads_v4::SquadsV4Program, InstructionParser},
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
    "3RbCdn9YDPs5xGN4vMJBmraeozC2LoBxrySoEVraWFKwQpty1dLtv8yRXUzYJoUTSmLZKjqbjLXNZ2coMA18NMYx";
const EXPECTED_MULTISIG: &str = "B6cxPRnWEdMfKuYspnZXDrPfnH2rSeht4VaoSjc1ibBm";
const EXPECTED_PROPOSAL: &str = "FSZZRGe9hnycwuK4HhhTNfLhYBV31txtK7Ut6WLuBASR";
const EXPECTED_TX_INDEX: u64 = 33;

fn pubkey_bytes(s: &str) -> Vec<u8> {
    Pubkey::from_str(s).unwrap().to_bytes().to_vec()
}

fn make_update() -> SubscribeUpdateTransaction {
    // Account layout from on-chain RPC response:
    //   0: 9d3hpFBcc8zowN9XMFh7ZeZb88VzUPv2uWMZuCgCNYx5  (payer/signer)
    //   1: DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL
    //   2: B6cxPRnWEdMfKuYspnZXDrPfnH2rSeht4VaoSjc1ibBm  (multisig)
    //   3: 12SNDKGbXeaL45kCuVrmpv8xWxrRqdLKGnTNoC62kDcg
    //   4: FSZZRGe9hnycwuK4HhhTNfLhYBV31txtK7Ut6WLuBASR  (proposal)
    //   5: ComputeBudget111111111111111111111111111111
    //   6: 11111111111111111111111111111111  (system program)
    //   7: SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf  (Squads V4)
    let account_keys = vec![
        pubkey_bytes("9d3hpFBcc8zowN9XMFh7ZeZb88VzUPv2uWMZuCgCNYx5"),
        pubkey_bytes("DttWaMuVvTiduZRnguLF7jNxTgiMBZ1hyAumKUiL2KRL"),
        pubkey_bytes("B6cxPRnWEdMfKuYspnZXDrPfnH2rSeht4VaoSjc1ibBm"),
        pubkey_bytes("12SNDKGbXeaL45kCuVrmpv8xWxrRqdLKGnTNoC62kDcg"),
        pubkey_bytes("FSZZRGe9hnycwuK4HhhTNfLhYBV31txtK7Ut6WLuBASR"),
        pubkey_bytes("ComputeBudget111111111111111111111111111111"),
        pubkey_bytes("11111111111111111111111111111111"),
        pubkey_bytes("SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf"),
    ];

    // ProposalCreate instruction:
    //   programIdIndex=7 (Squads V4), accounts=[2,4,0,0,6]
    //   data: anchor discriminator for "proposal_create"
    //       + transaction_index=33 as u64 little-endian
    //       + draft=false (0x00)
    let proposal_create_ix = CompiledInstruction {
        program_id_index: 7,
        accounts: vec![2, 4, 0, 0, 6],
        data: hex::decode("dc3c49e01e6c4f9f210000000000000000").unwrap(),
    };

    let sig_bytes = Signature::from_str(TX_SIG).unwrap().as_ref().to_vec();

    SubscribeUpdateTransaction {
        transaction: Some(SubscribeUpdateTransactionInfo {
            signature: sig_bytes.clone(),
            transaction: Some(Transaction {
                signatures: vec![sig_bytes],
                message: Some(Message {
                    account_keys,
                    instructions: vec![proposal_create_ix],
                    ..Default::default()
                }),
                ..Default::default()
            }),
            meta: Some(TransactionStatusMeta {
                err: None,
                log_messages: vec![
                    "Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf invoke [1]".to_string(),
                    "Program log: Instruction: ProposalCreate".to_string(),
                    "Program log: transaction index: 33".to_string(),
                    "Program SQDS4ep65T869zMMBKyuUq6aD6EgTu8psMjkvj52pCf success".to_string(),
                ],
                ..Default::default()
            }),
            ..Default::default()
        }),
        slot: 414508879,
    }
}

#[test]
fn squads_v4_proposal_create_fires_and_produces_slack_message() {
    let parser = JitoTransactionParser::new(make_update());

    let proposal_create = parser.instructions.iter().find_map(|ix| match ix {
        InstructionParser::SquadsV4(SquadsV4Program::ProposalCreate {
            multisig,
            proposal,
            transaction_index,
            draft,
        }) => Some((*multisig, *proposal, *transaction_index, *draft)),
        _ => None,
    });

    let (multisig, proposal, transaction_index, draft) =
        proposal_create.expect("expected a SquadsV4 ProposalCreate instruction");

    assert_eq!(multisig.to_string(), EXPECTED_MULTISIG);
    assert_eq!(proposal.to_string(), EXPECTED_PROPOSAL);
    assert_eq!(transaction_index, EXPECTED_TX_INDEX);
    assert!(!draft);
    assert_eq!(parser.transaction_signature, TX_SIG);

    // Build the Slack payload that would be posted to the webhook
    let ctx = SquadsContext::V4Proposal {
        multisig,
        proposal,
        transaction_index,
    };
    let payload = ctx.build_slack_payload(
        "Squads v4 proposal created",
        &parser.transaction_signature,
        "https://explorer.solana.com",
    );

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());

    // Header block
    assert_eq!(
        payload["blocks"][0]["text"]["text"],
        "Squads Proposal Created"
    );

    // Description block
    assert!(payload["blocks"][1]["text"]["text"]
        .as_str()
        .unwrap()
        .contains("Squads v4 proposal created"));

    // Fields block: multisig, proposal, tx index, and signature all appear
    let fields = payload["blocks"][2]["fields"].to_string();
    assert!(fields.contains(EXPECTED_MULTISIG));
    assert!(fields.contains(EXPECTED_PROPOSAL));
    assert!(fields.contains(&EXPECTED_TX_INDEX.to_string()));
    assert!(fields.contains(TX_SIG));
}
