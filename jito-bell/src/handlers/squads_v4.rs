//! Squads v4 instruction notification handling.

use crate::{
    error::JitoBellError, handlers::squads_common::SquadsContext,
    ix_parser::squads_v4::SquadsV4Program, program::InstructionConfig,
    tx_parser::JitoTransactionParser, JitoBellHandler,
};

/// Sends simple amount-less alerts for configured Squads v4 proposal create instructions.
pub(crate) async fn handle_squads_v4_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    squads_v4_instruction: &SquadsV4Program,
    instruction: &InstructionConfig,
) -> Result<(), JitoBellError> {
    if let (
        SquadsV4Program::ProposalCreate {
            multisig,
            proposal,
            transaction_index,
            ..
        },
        Some(notification_info),
    ) = (squads_v4_instruction, &instruction.notification_info)
    {
        let squads_context = SquadsContext::V4Proposal {
            multisig: *multisig,
            proposal: *proposal,
            transaction_index: *transaction_index,
        };
        handler
            .dispatch_slack(
                &notification_info.description,
                &parser.transaction_signature,
                squads_context,
                &notification_info.destinations,
            )
            .await?;
    }

    Ok(())
}
