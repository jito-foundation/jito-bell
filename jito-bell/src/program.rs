use std::collections::HashMap;

use serde::Deserialize;

use crate::{
    notification_info::Destination,
    threshold_config::{ThresholdConfig, UsdThresholdConfig},
};

#[derive(Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProgramName {
    JitoSteward,
    SplToken2022,
    SplStakePool,
    JitoVault,
}

#[derive(Deserialize)]
pub struct Program {
    /// Program ID
    pub program_id: String,

    /// Instructions configurations
    #[serde(default)]
    pub instructions: HashMap<String, Instruction>,

    /// Events configurations
    #[serde(default)]
    pub events: HashMap<String, EventConfig>,
}

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

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum EventConfig {
    // Events with thresholds (like rebalance)
    WithThresholds {
        thresholds: Vec<ThresholdConfig>,
    },

    // Simple events without thresholds
    Simple {
        destinations: Vec<Destination>,
        description: String,
    },
}
