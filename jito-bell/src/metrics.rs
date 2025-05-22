#[derive(Debug, Default)]
pub(crate) struct NotificationMetrics {
    pub(crate) success: u64,
    pub(crate) fail: u64,
}

#[derive(Debug, Default)]
pub(crate) struct EpochMetrics {
    /// Current Epoch
    pub(crate) epoch: u64,

    /// Transaction Metrics
    pub(crate) tx: u64,

    /// Notification Metrics
    pub(crate) notification: NotificationMetrics,
}

impl EpochMetrics {
    pub fn new(epoch: u64) -> Self {
        Self {
            epoch,
            ..Default::default()
        }
    }

    pub fn increment_tx_count(&mut self) {
        self.tx += 1;
    }

    pub fn increment_success_notification_count(&mut self) {
        self.notification.success += 1;
    }

    pub fn increment_fail_notification_count(&mut self) {
        self.notification.fail += 1;
    }
}
