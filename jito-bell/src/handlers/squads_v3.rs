//! Squads v3 instruction notification handling.

use log::debug;

use crate::{
    error::JitoBellError, handlers::squads_common::SquadsContext,
    ix_parser::squads_v3::SquadsV3Program, program::InstructionConfig,
    tx_parser::JitoTransactionParser, JitoBellHandler,
};

/// Sends simple amount-less alerts for configured Squads v3 create instructions.
pub(crate) async fn handle_squads_v3_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    squads_v3_instruction: &SquadsV3Program,
    instruction: &InstructionConfig,
) -> Result<(), JitoBellError> {
    debug!("Squads v3 Instruction: {squads_v3_instruction}");

    if let (
        SquadsV3Program::CreateTransaction {
            multisig,
            transaction,
            ..
        },
        Some(notification_info),
    ) = (squads_v3_instruction, &instruction.notification_info)
    {
        let squads_context = SquadsContext::V3Transaction {
            multisig: *multisig,
            transaction: *transaction,
        };
        handler
            .dispatch_slack(
                &notification_info.description,
                &parser.transaction_signature,
                squads_context,
            )
            .await?;
    }

    Ok(())
}
