use serde::Deserialize;

use crate::threshold_config::{ThresholdConfig, UsdThresholdConfig};

#[derive(Deserialize, Debug)]
pub struct Instruction {
    /// Stake Pool address
    pub stake_pool: Option<String>,

    /// Pool mint token address
    pub pool_mint: Option<String>,

    /// Vault Receipt token address
    pub vrt: Option<String>,

    /// Thresholds (replaces the single threshold)
    #[serde(default)]
    pub thresholds: Vec<ThresholdConfig>,

    /// Thresholds (replaces the single threshold)
    #[serde(default)]
    pub usd_thresholds: Vec<UsdThresholdConfig>,
}
