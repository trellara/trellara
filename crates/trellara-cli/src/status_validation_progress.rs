use trellara_checkpoint::parse_lsn;
use trellara_checkpoint::CheckpointLag;
use trellara_checkpoint::ValidationEvent;

use crate::FlowStatusSummary;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValidationProgress {
    pub(crate) is_current: bool,
    pub(crate) source_lag_bytes: Option<u64>,
    pub(crate) target_lag_bytes: Option<u64>,
}

impl ValidationProgress {
    pub(crate) fn from_status(status: &FlowStatusSummary) -> Self {
        let Some(validation) = status.latest_validation.as_ref() else {
            return Self::missing();
        };
        Self::from_parts(validation, status.source.as_ref(), status.target.as_ref())
    }

    pub(crate) fn from_parts(
        validation: &ValidationEvent,
        source: Option<&CheckpointLag>,
        target: Option<&CheckpointLag>,
    ) -> Self {
        let Some(source) = source else {
            return Self::missing();
        };
        let Some(target) = target else {
            return Self::missing();
        };

        let validation_source_lsn = parse_lsn(&validation.source_watermark_lsn);
        let validation_target_lsn = parse_lsn(&validation.target_watermark_lsn);
        let source_durable_lsn = parse_lsn(&source.last_durable_lsn);
        let target_applied_lsn = parse_lsn(&target.last_applied_lsn);

        Self {
            is_current: validation_source_lsn >= source_durable_lsn
                && validation_target_lsn >= target_applied_lsn,
            source_lag_bytes: Some(source_durable_lsn.saturating_sub(validation_source_lsn)),
            target_lag_bytes: Some(target_applied_lsn.saturating_sub(validation_target_lsn)),
        }
    }

    fn missing() -> Self {
        Self {
            is_current: false,
            source_lag_bytes: None,
            target_lag_bytes: None,
        }
    }
}

pub(crate) fn validation_stale_message(progress: ValidationProgress) -> String {
    format!(
        "latest validation checksums match but validation watermarks are stale by source={} bytes target={} bytes",
        progress.source_lag_bytes.unwrap_or_default(),
        progress.target_lag_bytes.unwrap_or_default()
    )
}
