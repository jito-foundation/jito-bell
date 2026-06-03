//! Squads v3 instruction notification handling.

use log::debug;

use crate::{
    error::JitoBellError, ix_parser::squads_v3::SquadsV3Program, program::Instruction,
    tx_parser::JitoTransactionParser, JitoBellHandler,
};

/// Sends simple amount-less alerts for configured Squads v3 create instructions.
pub(crate) async fn handle_squads_v3_program(
    handler: &mut JitoBellHandler,
    parser: &JitoTransactionParser,
    squads_v3_instruction: &SquadsV3Program,
    instruction: &Instruction,
) -> Result<(), JitoBellError> {
    debug!("Squads v3 Instruction: {squads_v3_instruction}");

    if !matches!(
        squads_v3_instruction,
        SquadsV3Program::CreateTransaction { ix: _ }
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
