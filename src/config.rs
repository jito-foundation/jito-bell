use std::collections::HashMap;

use serde::Deserialize;

use crate::{notification_config::NotificationConfig, program::Program};

#[derive(Deserialize)]
pub struct JitoBellConfig {
    /// Programs Configuration
    pub programs: HashMap<String, Program>,

    /// Notifications Configuration
    pub notifications: NotificationConfig,

    /// Block explorer url
    pub explorer_url: String,

    /// Message Templates
    pub message_templates: HashMap<String, String>,
}

impl std::fmt::Display for JitoBellConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Program:")?;
        for program in self.programs.values() {
            writeln!(f, "  Program ID: {}", program.program_id)?;

            writeln!(f, "  Instruction")?;
            for (key, instruction) in program.instructions.iter() {
                writeln!(f, "    Instruction: {}", key)?;

                if let Some(pool_mint_address) = &instruction.pool_mint {
                    writeln!(f, "        Pool Mint: {}", pool_mint_address)?;
                }

                if let Some(vrt_address) = &instruction.vrt {
                    writeln!(f, "        VRT: {}", vrt_address)?;
                }

                writeln!(f, "        Thresholds")?;
                for threshold in instruction.thresholds.iter() {
                    writeln!(f, "           Threshold Value: {}", threshold.value)?;
                    writeln!(f, "           Notification")?;
                    writeln!(
                        f,
                        "               Description: {}",
                        threshold.notification.description
                    )?;

                    let destinations = threshold.notification.destinations.join(",");
                    writeln!(f, "               Destinations: {}", destinations)?;
                }

                writeln!(f, "        USD Thresholds")?;
                for threshold in instruction.usd_thresholds.iter() {
                    writeln!(f, "           USD Threshold Value: {}", threshold.value)?;
                    writeln!(f, "           Notification")?;
                    writeln!(
                        f,
                        "               Description: {}",
                        threshold.notification.description
                    )?;

                    let destinations = threshold.notification.destinations.join(",");
                    writeln!(f, "               Destinations: {}", destinations)?;
                }
            }
        }

        Ok(())
    }
}
