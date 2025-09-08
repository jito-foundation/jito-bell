use std::collections::HashMap;

use serde::Deserialize;

use crate::program::Program;

#[derive(Deserialize)]
pub struct JitoBellConfig {
    /// Programs Configuration
    pub programs: HashMap<String, Program>,

    // Notifications Configuration
    // pub notifications: NotificationConfig,
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

                if let Some(lsts) = &instruction.lsts {
                    for (lst_address, alert_config) in lsts.iter() {
                        writeln!(f, "        Pool Mint: {}", lst_address)?;

                        writeln!(f, "        Thresholds")?;
                        for threshold in alert_config.thresholds.iter() {
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
                    }
                }

                if let Some(vrts) = &instruction.vrts {
                    for (vrt_address, config) in vrts.iter() {
                        writeln!(f, "        VRT: {}", vrt_address)?;

                        for threshold in config.thresholds.iter() {
                            writeln!(f, "           VRT Threshold Value: {}", threshold.value)?;
                        }

                        for threshold in config.usd_thresholds.iter() {
                            writeln!(f, "           VRT USD Threshold Value: {}", threshold.value)?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
