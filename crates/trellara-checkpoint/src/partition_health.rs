use serde::{Deserialize, Serialize};

use crate::partition_health_actions::partition_visibility_actions;
use crate::partition_health_watermarks::{
    global_low_watermark_partition_ids, lsn_skew, max_lsn, partition_stragglers,
};
use crate::PartitionWatermarkSummary;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PartitionScaleHealthStatus {
    MissingPartitions,
    LaggingPartitions,
    Ready,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionScaleHealthSummary {
    pub source_id: String,
    pub dataset_id: String,
    pub expected_partition_count: u32,
    pub observed_partition_count: u32,
    pub status: PartitionScaleHealthStatus,
    pub global_watermark_available: bool,
    pub global_visibility_releasable: bool,
    pub global_durable_lsn: Option<String>,
    pub global_applied_lsn: Option<String>,
    pub max_observed_durable_lsn: Option<String>,
    pub max_observed_applied_lsn: Option<String>,
    pub observed_durable_skew_bytes: Option<u64>,
    pub observed_applied_skew_bytes: Option<u64>,
    pub max_partition_lag_bytes: Option<u64>,
    pub global_visibility_blocker_codes: Vec<String>,
    pub global_durable_low_watermark_partition_ids: Vec<u32>,
    pub global_applied_low_watermark_partition_ids: Vec<u32>,
    pub lagging_partition_ids: Vec<u32>,
    pub straggler_partition_ids: Vec<u32>,
    pub blocking_partition_ids: Vec<u32>,
    pub missing_partitions: Vec<u32>,
    pub visibility_actions: Vec<PartitionScaleHealthAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PartitionScaleHealthAction {
    pub code: String,
    pub partition_ids: Vec<u32>,
    pub command: String,
    pub reason: String,
}

impl PartitionScaleHealthSummary {
    pub fn from_watermark_summary(summary: &PartitionWatermarkSummary) -> Self {
        let lagging_partition_ids = summary
            .partitions
            .iter()
            .filter(|partition| partition.durable_to_applied_bytes > 0)
            .map(|partition| partition.partition_id)
            .collect::<Vec<_>>();
        let blocking_partition_ids = summary
            .partitions
            .iter()
            .filter(|partition| partition.blocks_global_applied_watermark)
            .map(|partition| partition.partition_id)
            .collect::<Vec<_>>();
        let max_observed_applied_lsn = max_lsn(
            summary
                .partitions
                .iter()
                .map(|partition| &partition.last_applied_lsn),
        );
        let global_durable_low_watermark_partition_ids = global_low_watermark_partition_ids(
            summary,
            summary.global_durable_lsn.as_ref(),
            |partition| &partition.last_durable_lsn,
        );
        let global_applied_low_watermark_partition_ids = global_low_watermark_partition_ids(
            summary,
            summary.global_applied_lsn.as_ref(),
            |partition| &partition.last_applied_lsn,
        );
        let straggler_partition_ids =
            partition_stragglers(summary, max_observed_applied_lsn.as_ref());
        let status =
            partition_scale_status(summary, &lagging_partition_ids, &blocking_partition_ids);
        let global_visibility_releasable =
            status == PartitionScaleHealthStatus::Ready && blocking_partition_ids.is_empty();
        let visibility_actions = partition_visibility_actions(
            summary,
            &status,
            &blocking_partition_ids,
            &straggler_partition_ids,
        );
        let global_visibility_blocker_codes = global_visibility_blocker_codes(
            summary,
            &lagging_partition_ids,
            &blocking_partition_ids,
        );

        Self {
            source_id: summary.source_id.clone(),
            dataset_id: summary.dataset_id.clone(),
            expected_partition_count: summary.expected_partition_count,
            observed_partition_count: summary.observed_partition_count,
            status,
            global_watermark_available: summary.complete_partition_set,
            global_visibility_releasable,
            global_durable_lsn: summary.global_durable_lsn.clone(),
            global_applied_lsn: summary.global_applied_lsn.clone(),
            max_observed_durable_lsn: max_lsn(
                summary
                    .partitions
                    .iter()
                    .map(|partition| &partition.last_durable_lsn),
            ),
            max_observed_applied_lsn,
            observed_durable_skew_bytes: lsn_skew(
                summary
                    .partitions
                    .iter()
                    .map(|partition| &partition.last_durable_lsn),
            ),
            observed_applied_skew_bytes: lsn_skew(
                summary
                    .partitions
                    .iter()
                    .map(|partition| &partition.last_applied_lsn),
            ),
            max_partition_lag_bytes: summary
                .partitions
                .iter()
                .map(|partition| partition.durable_to_applied_bytes)
                .max(),
            global_visibility_blocker_codes,
            global_durable_low_watermark_partition_ids,
            global_applied_low_watermark_partition_ids,
            lagging_partition_ids,
            straggler_partition_ids,
            blocking_partition_ids,
            missing_partitions: summary.missing_partitions.clone(),
            visibility_actions,
        }
    }
}

fn global_visibility_blocker_codes(
    summary: &PartitionWatermarkSummary,
    lagging_partition_ids: &[u32],
    blocking_partition_ids: &[u32],
) -> Vec<String> {
    let mut codes = Vec::new();
    if !summary.missing_partitions.is_empty() {
        codes.push("missing_partition_watermarks".to_string());
    }
    if !blocking_partition_ids.is_empty() {
        codes.push("global_applied_low_watermark_blocked".to_string());
    }
    if !lagging_partition_ids.is_empty() {
        codes.push("partition_durable_apply_lag".to_string());
    }
    codes
}

fn partition_scale_status(
    summary: &PartitionWatermarkSummary,
    lagging_partition_ids: &[u32],
    blocking_partition_ids: &[u32],
) -> PartitionScaleHealthStatus {
    if !summary.complete_partition_set {
        PartitionScaleHealthStatus::MissingPartitions
    } else if !lagging_partition_ids.is_empty() || !blocking_partition_ids.is_empty() {
        PartitionScaleHealthStatus::LaggingPartitions
    } else {
        PartitionScaleHealthStatus::Ready
    }
}
