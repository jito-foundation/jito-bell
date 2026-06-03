//! Squads v4 instruction notification handling.

use log::debug;

use crate::{
    error::JitoBellError, ix_parser::squads_v4::SquadsV4Program, program::Instruction,
    tx_parser::JitoTransactionParser, JitoBellHandler,
};

/// Sends simple amount-less alerts for configured Squads v4 proposal create instructions.
pub(crate) async fn handle_squads_v4_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    squads_v4_instruction: &SquadsV4Program,
    instruction: &Instruction,
) -> Result<(), JitoBellError> {
    debug!("Squads v4 Instruction: {squads_v4_instruction}");

    if !matches!(
        squads_v4_instruction,
        SquadsV4Program::ProposalCreate { ix: _ }
    ) {
        return Ok(());
    }

    if let Some(ref notification_info) = instruction.notification_info {
        handler
            .dispatch_platform_notifications(
                &notification_info.destinations,
                &notification_info.description,
                None,
                None,
                &parser.transaction_signature,
            )
            .await?;
    }

    Ok(())
}
