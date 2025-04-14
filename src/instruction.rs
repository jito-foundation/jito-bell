use serde::Deserialize;

use crate::threshold_config::ThresholdConfig;

#[derive(Deserialize, Debug)]
pub struct Instruction {
    /// Pool mint token address
    pub pool_mint: String,

    /// Thresholds (replaces the single threshold)
    #[serde(default)]
    pub thresholds: Vec<ThresholdConfig>,
}
