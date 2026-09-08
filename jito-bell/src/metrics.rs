use log::info;
use solana_metrics::{datapoint::DataPoint, datapoint_info, submit};
use solana_sdk::clock::DEFAULT_SLOTS_PER_EPOCH;

#[derive(Debug, Default)]
pub(crate) struct NotificationMetrics {
    pub(crate) success: u64,
    pub(crate) fail: u64,
}

/// Per-epoch counters for the Squads monitoring pipeline.
/// Each counter corresponds to a distinct failure or processing stage so gaps
/// between `proposals_parsed` and successfully-sent notifications are visible.
#[derive(Debug, Default)]
pub(crate) struct SquadsMetrics {
    /// V3 CreateTransaction or V4 ProposalCreate instructions successfully parsed.
    pub(crate) proposals_parsed: u64,
    /// Squads discriminator matched but inner argument/account parsing returned None.
    pub(crate) parse_errors: u64,
    /// Parsed instruction had no matching handler config entry.
    pub(crate) no_config: u64,
    /// dispatch_slack called but no webhook URLs resolved from the destination list.
    pub(crate) no_webhook: u64,
    /// A Squads webhook dispatch had at least one delivery failure.
    /// This is the single alert surface for both partial and total webhook failure.
    pub(crate) webhook_failure: u64,
    /// Total individual webhook HTTP/transport errors across all dispatches.
    pub(crate) webhook_errors: u64,
}

#[derive(Debug, Default)]
pub(crate) struct EpochMetrics {
    /// Current Epoch
    pub(crate) epoch: u64,

    /// Most recent observed slot
    pub(crate) slot: u64,

    /// Transactions received from the stream
    pub(crate) tx: u64,

    /// Transactions that were on-chain failures (meta.err set); skipped for notification.
    pub(crate) failed_tx: u64,

    /// Notification Metrics
    pub(crate) notification: NotificationMetrics,

    /// Squads-specific pipeline metrics
    pub(crate) squads: SquadsMetrics,
}

impl EpochMetrics {
    pub fn new(epoch: u64, slot: u64) -> Self {
        Self {
            epoch,
            slot,
            ..Default::default()
        }
    }

    pub fn update_slot(&mut self, slot: u64) {
        self.slot = slot;
    }

    fn emit_live_metric(&self, name: &'static str, count: u64) {
        datapoint_info!(name, ("count", count, i64), ("epoch", self.epoch, i64),);
    }

    pub fn emit_epoch_progress(&self, slot: u64) {
        if slot.is_multiple_of(DEFAULT_SLOTS_PER_EPOCH / 10) {
            let position = slot % DEFAULT_SLOTS_PER_EPOCH;
            info!(
                "epoch={} slot={} ({position}/{}) tx={} failed_tx={} notif_ok={} notif_fail={} squads_parsed={}",
                self.epoch,
                slot,
                DEFAULT_SLOTS_PER_EPOCH,
                self.tx,
                self.failed_tx,
                self.notification.success,
                self.notification.fail,
                self.squads.proposals_parsed,
            );
        }
    }

    pub fn emit_slot_heartbeat(&self, slot: u64) {
        if slot.is_multiple_of(100) {
            // Manual datapoint construction (vs. datapoint_info!): we want the point
            // submitted to InfluxDB unconditionally, with only the agent's per-point
            // log line filtered out by RUST_LOG. datapoint_debug! would gate the
            // submit on log_enabled!(Debug), dropping the metric at default verbosity.
            let mut point = DataPoint::new("jito-bell-slot-heartbeat");
            point.add_field_i64("slot", slot as i64);
            point.add_field_i64("epoch", self.epoch as i64);
            submit(point, log::Level::Debug);
        }
    }

    pub fn increment_tx_count(&mut self) {
        self.tx += 1;
        self.emit_live_metric("jito-bell-transactions", 1);
    }

    pub fn increment_failed_tx_count(&mut self) {
        self.failed_tx += 1;
        self.emit_live_metric("jito-bell-failed-transactions", 1);
    }

    pub fn increment_success_notification_count(&mut self) {
        self.notification.success += 1;
        self.emit_live_metric("jito-bell-success-notifications", 1);
    }

    pub fn increment_fail_notification_count(&mut self) {
        self.notification.fail += 1;
        self.emit_live_metric("jito-bell-failed-notifications", 1);
    }

    pub fn increment_squads_parsed(&mut self) {
        self.squads.proposals_parsed += 1;
        self.emit_live_metric("jito-bell-squads-proposals-parsed", 1);
    }

    pub fn increment_squads_parse_errors(&mut self, count: u64) {
        if count == 0 {
            return;
        }
        self.squads.parse_errors += count;
        self.emit_live_metric("jito-bell-squads-parse-errors", count);
    }

    pub fn increment_squads_no_config(&mut self) {
        self.squads.no_config += 1;
        self.emit_live_metric("jito-bell-squads-no-config", 1);
    }

    pub fn increment_squads_no_webhook(&mut self) {
        self.squads.no_webhook += 1;
        self.emit_live_metric("jito-bell-squads-no-webhook", 1);
    }

    pub fn increment_squads_webhook_failure(&mut self) {
        self.squads.webhook_failure += 1;
        self.emit_live_metric("jito-bell-squads-webhook-failure", 1);
    }

    pub fn increment_squads_webhook_errors(&mut self) {
        self.squads.webhook_errors += 1;
        self.emit_live_metric("jito-bell-squads-webhook-errors", 1);
    }
}
