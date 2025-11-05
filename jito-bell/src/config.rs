use std::collections::HashMap;

use serde::Deserialize;

use crate::program::{Program, ProgramName};

#[derive(Deserialize)]
pub struct JitoBellConfig {
    /// Programs Configuration
    pub programs: HashMap<ProgramName, Program>,

    /// Block explorer url
    pub explorer_url: String,

    /// Message Templates
    pub message_templates: HashMap<String, String>,
}

impl JitoBellConfig {
    /// Get a message template by name, falling back to default
    pub fn get_template(&self, name: &str) -> Option<&String> {
        self.message_templates
            .get(name)
            .or_else(|| self.message_templates.get("default"))
    }
}

impl std::fmt::Display for JitoBellConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Programs:")?;
        for program in self.programs.values() {
            writeln!(f, "  Program ID: {}", program.program_id)?;

            if !program.instructions.is_empty() {
                writeln!(f, "  Instructions:")?;
                for (key, instruction) in program.instructions.iter() {
                    writeln!(f, "    Instruction: {}", key)?;

                    if let Some(lsts) = &instruction.lsts {
                        for (lst_address, alert_config) in lsts.iter() {
                            writeln!(f, "      Pool Mint: {}", lst_address)?;
                            writeln!(f, "      Thresholds:")?;
                            for threshold in alert_config.thresholds.iter() {
                                writeln!(f, "        Threshold Value: {}", threshold.value)?;
                                writeln!(f, "        Notification:")?;
                                writeln!(
                                    f,
                                    "          Description: {}",
                                    threshold.notification.description
                                )?;
                                let destinations = threshold
                                    .notification
                                    .destinations
                                    .iter()
                                    .map(|d| d.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ");
                                writeln!(f, "          Destinations: {}", destinations)?;
                            }
                        }
                    }

                    if let Some(vrts) = &instruction.vrts {
                        for (vrt_address, config) in vrts.iter() {
                            writeln!(f, "      VRT: {}", vrt_address)?;
                            if !config.thresholds.is_empty() {
                                writeln!(f, "        VRT Thresholds:")?;
                                for threshold in config.thresholds.iter() {
                                    writeln!(f, "          Value: {}", threshold.value)?;
                                }
                            }
                            if !config.usd_thresholds.is_empty() {
                                writeln!(f, "        USD Thresholds:")?;
                                for threshold in config.usd_thresholds.iter() {
                                    writeln!(f, "          Value: ${}", threshold.value)?;
                                }
                            }
                        }
                    }

                    if let Some(stake_pools) = &instruction.stake_pools {
                        for (pool_address, alert_config) in stake_pools.iter() {
                            writeln!(f, "      Stake Pool: {}", pool_address)?;
                            writeln!(f, "      Thresholds:")?;
                            for threshold in alert_config.thresholds.iter() {
                                writeln!(f, "        Value: {}", threshold.value)?;
                            }
                        }
                    }
                }
            }

            if !program.events.is_empty() {
                writeln!(f, "  Events:")?;
                for (key, event) in program.events.iter() {
                    writeln!(f, "    Event: {}", key)?;
                    writeln!(f, "      Description: {}", event.description)?;
                    let destinations = event
                        .destinations
                        .iter()
                        .map(|d| d.to_string())
                        .collect::<Vec<String>>()
                        .join(", ");
                    writeln!(f, "      Destinations: {}", destinations)?;
                }
            }
        }
        Ok(())
    }
}
