use std::collections::HashMap;

use serde::Deserialize;

use crate::threshold_config::{ThresholdConfig, UsdThresholdConfig};

#[derive(Debug, Clone, Deserialize)]
pub struct AlertConfig {
    /// Thresholds (replaces the single threshold)
    #[serde(default)]
    pub thresholds: Vec<ThresholdConfig>,

    /// Thresholds (replaces the single threshold)
    #[serde(default)]
    pub usd_thresholds: Vec<UsdThresholdConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Instruction {
    /// Stake Pool
    pub stake_pools: Option<HashMap<String, AlertConfig>>,

    /// Pool mint token (LST)
    pub lsts: Option<HashMap<String, AlertConfig>>,

    /// Vault receipt token (VRT)
    pub vrts: Option<HashMap<String, AlertConfig>>,
}
