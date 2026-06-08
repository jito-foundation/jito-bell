use std::collections::HashMap;
use std::str::FromStr;

use serde::{Deserialize, Deserializer};
use solana_sdk::pubkey::Pubkey;

use crate::program::{EventConfig, ProgramConfig, ProgramName};

#[derive(Deserialize)]
pub struct JitoBellConfig {
    /// Yellowstone transaction subscription filters.
    pub filters: FilterOptions,

    /// Programs Configuration
    pub programs: HashMap<ProgramName, ProgramConfig>,

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

#[derive(Debug, Clone, Deserialize)]
pub struct FilterOptions {
    /// Filter vote transactions
    pub vote: Option<bool>,

    /// Filter failed transactions
    pub failed: Option<bool>,

    /// Filter by transaction signature
    pub signature: Option<String>,

    /// Filter included accounts in transactions
    #[serde(
        default,
        deserialize_with = "deserialize_pubkey_vec",
        rename = "accounts-include",
        alias = "accounts_include"
    )]
    pub accounts_include: Vec<Pubkey>,

    /// Filter excluded accounts in transactions
    #[serde(
        default,
        deserialize_with = "deserialize_pubkey_vec",
        rename = "accounts-exclude",
        alias = "accounts_exclude"
    )]
    pub accounts_exclude: Vec<Pubkey>,

    /// Filter required accounts in transactions
    #[serde(
        default,
        deserialize_with = "deserialize_pubkey_vec",
        rename = "accounts-required",
        alias = "accounts_required"
    )]
    pub accounts_required: Vec<Pubkey>,
}

fn deserialize_pubkey_vec<'de, D>(deserializer: D) -> Result<Vec<Pubkey>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<String>::deserialize(deserializer)?
        .into_iter()
        .map(|value| {
            Pubkey::from_str(&value).map_err(|error| {
                serde::de::Error::custom(format!("invalid pubkey {value}: {error}"))
            })
        })
        .collect()
}

impl std::fmt::Display for JitoBellConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Explorer URL: {}", self.explorer_url)?;

        writeln!(f, "Filters:")?;
        if let Some(vote) = self.filters.vote {
            writeln!(f, "  Vote: {}", vote)?;
        }
        if let Some(failed) = self.filters.failed {
            writeln!(f, "  Failed: {}", failed)?;
        }
        if let Some(signature) = &self.filters.signature {
            writeln!(f, "  Signature: {}", signature)?;
        }
        if !self.filters.accounts_include.is_empty() {
            writeln!(f, "  Accounts Include:")?;
            for account in &self.filters.accounts_include {
                writeln!(f, "    - {}", account)?;
            }
        }
        if !self.filters.accounts_exclude.is_empty() {
            writeln!(f, "  Accounts Exclude:")?;
            for account in &self.filters.accounts_exclude {
                writeln!(f, "    - {}", account)?;
            }
        }
        if !self.filters.accounts_required.is_empty() {
            writeln!(f, "  Accounts Required:")?;
            for account in &self.filters.accounts_required {
                writeln!(f, "    - {}", account)?;
            }
        }

        writeln!(f, "Message Templates:")?;
        for (name, template) in &self.message_templates {
            writeln!(f, "  {}: {}", name, template)?;
        }

        writeln!(f, "Programs:")?;
        for (program_name, program) in &self.programs {
            let program_name = match program_name {
                ProgramName::JitoSteward => "jito_steward",
                ProgramName::SplToken2022 => "spl_token2022",
                ProgramName::SplStakePool => "spl_stake_pool",
                ProgramName::JitoVault => "jito_vault",
                ProgramName::SquadsV3 => "squads_v3",
                ProgramName::SquadsV4 => "squads_v4",
            };
            writeln!(f, "  Program Name: {}", program_name)?;
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
                for (key, event_config) in program.events.iter() {
                    writeln!(f, "    Event: {}", key)?;

                    match event_config {
                        EventConfig::WithThresholds { thresholds } => {
                            writeln!(f, "      Type: Threshold-based")?;
                            writeln!(f, "      Thresholds:")?;
                            for (idx, threshold) in thresholds.iter().enumerate() {
                                writeln!(f, "        [{}] Value: {}", idx + 1, threshold.value)?;
                                writeln!(
                                    f,
                                    "            Description: {}",
                                    threshold.notification.description
                                )?;
                                let destinations = threshold
                                    .notification
                                    .destinations
                                    .iter()
                                    .map(|d| d.to_string())
                                    .collect::<Vec<String>>()
                                    .join(", ");
                                writeln!(f, "            Destinations: {}", destinations)?;
                            }
                        }
                        EventConfig::Simple {
                            destinations,
                            description,
                        } => {
                            writeln!(f, "      Type: Simple")?;
                            writeln!(f, "      Description: {}", description)?;
                            let dests = destinations
                                .iter()
                                .map(|d| d.to_string())
                                .collect::<Vec<String>>()
                                .join(", ");
                            writeln!(f, "      Destinations: {}", dests)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
