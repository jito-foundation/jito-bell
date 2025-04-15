use serde::Deserialize;

use crate::threshold_config::ThresholdConfig;

#[derive(Deserialize, Debug)]
pub struct Instruction {
    /// Pool mint token address
    pub pool_mint: Option<String>,

    /// Vault Receipt token address
    pub vrt: Option<String>,

    /// Thresholds (replaces the single threshold)
    #[serde(default)]
    pub thresholds: Vec<ThresholdConfig>,
}
