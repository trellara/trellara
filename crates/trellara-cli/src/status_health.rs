use serde::Serialize;
use trellara_checkpoint::{
    ApplyQuarantine, CheckpointLag, PartitionWatermarkSummary, ValidationEvent,
};
use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};

use crate::{status_health_issues::evaluate_flow_health, status_health_status::flow_health_status};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowHealthSummary {
    pub(crate) status: FlowHealthStatus,
    pub(crate) issue_count: usize,
    pub(crate) issues: Vec<String>,
}

pub(crate) struct FlowHealthParts<'a> {
    pub(crate) source_slot: &'a ReplicationSlotStatus,
    pub(crate) subscription_conflicts: &'a [SubscriptionConflictStats],
    pub(crate) source_wal_retention_warn_bytes: Option<i64>,
    pub(crate) source: Option<&'a CheckpointLag>,
    pub(crate) target: Option<&'a CheckpointLag>,
    pub(crate) partition_watermarks: Option<&'a PartitionWatermarkSummary>,
    pub(crate) latest_quarantine: Option<&'a ApplyQuarantine>,
    pub(crate) latest_validation: Option<&'a ValidationEvent>,
}

impl FlowHealthSummary {
    pub(crate) fn from_parts(parts: FlowHealthParts<'_>) -> Self {
        let evaluation = evaluate_flow_health(&parts);
        let status = flow_health_status(
            &parts,
            evaluation.issues.is_empty(),
            evaluation.subscription_conflict_blocked,
        );

        Self {
            status,
            issue_count: evaluation.issues.len(),
            issues: evaluation.issues,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum FlowHealthStatus {
    Healthy,
    Degraded,
    Blocked,
}
