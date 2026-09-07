use trellara_checkpoint::parse_lsn;
use trellara_checkpoint::CheckpointLag;

use crate::FlowStatusSummary;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct TargetSourceProgress {
    pub(crate) reaches_source_durable: bool,
    pub(crate) source_to_target_lag_bytes: Option<u64>,
}

impl TargetSourceProgress {
    pub(crate) fn from_status(status: &FlowStatusSummary) -> Self {
        Self::from_lags(status.source.as_ref(), status.target.as_ref())
    }

    pub(crate) fn from_lags(
        source: Option<&CheckpointLag>,
        target: Option<&CheckpointLag>,
    ) -> Self {
        let Some(source) = source else {
            return Self::missing();
        };
        let Some(target) = target else {
            return Self::missing();
        };

        let source_durable_lsn = parse_lsn(&source.last_durable_lsn);
        let target_applied_lsn = parse_lsn(&target.last_applied_lsn);
        let source_to_target_lag_bytes = source_durable_lsn.saturating_sub(target_applied_lsn);

        Self {
            reaches_source_durable: target.target_is_caught_up
                && target_applied_lsn >= source_durable_lsn,
            source_to_target_lag_bytes: Some(source_to_target_lag_bytes),
        }
    }

    fn missing() -> Self {
        Self {
            reaches_source_durable: false,
            source_to_target_lag_bytes: None,
        }
    }
}
