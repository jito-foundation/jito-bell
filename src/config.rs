use std::collections::HashMap;

use serde::Deserialize;

use crate::{notification_config::NotificationConfig, program::Program};

#[derive(Deserialize)]
pub struct JitoBellConfig {
    /// Programs Configuration
    pub programs: HashMap<String, Program>,

    /// Notifications Configuration
    pub notifications: NotificationConfig,

    /// Message Templates
    pub message_templates: HashMap<String, String>,
}

impl std::fmt::Display for JitoBellConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program:")?;
        for program in self.programs.values() {
            writeln!(f, "  Program ID: {}", program.program_id)?;

            writeln!(f, "  Instruction")?;
            for instruction in program.instructions.values() {
                writeln!(f, "    Pool Mint: {}", instruction.pool_mint)?;
                writeln!(f, "    Threshold: {}", instruction.threshold)?;

                writeln!(f, "    Notification")?;
                writeln!(
                    f,
                    "    Description: {}",
                    instruction.notification.description
                )?;

                let destinations = instruction.notification.destinations.join(",");
                writeln!(f, "    Destinations: {}", destinations)?;
            }
        }

        Ok(())
    }
}
