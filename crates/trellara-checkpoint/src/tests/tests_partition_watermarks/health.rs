use super::*;

#[test]
fn partition_scale_health_reports_ready_partitions() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        2,
        vec![
            partition_checkpoint(0, "0/16B9000", "0/16B9000"),
            partition_checkpoint(1, "0/16B9000", "0/16B9000"),
        ],
    )
    .expect("watermark summary");

    let health = PartitionScaleHealthSummary::from_watermark_summary(&summary);

    assert_eq!(health.status, PartitionScaleHealthStatus::Ready);
    assert!(health.global_watermark_available);
    assert!(health.global_visibility_releasable);
    assert_eq!(health.global_durable_lsn, Some("0/16B9000".to_string()));
    assert_eq!(health.global_applied_lsn, Some("0/16B9000".to_string()));
    assert_eq!(
        health.max_observed_durable_lsn,
        Some("0/16B9000".to_string())
    );
    assert_eq!(
        health.max_observed_applied_lsn,
        Some("0/16B9000".to_string())
    );
    assert_eq!(health.observed_durable_skew_bytes, Some(0));
    assert_eq!(health.observed_applied_skew_bytes, Some(0));
    assert_eq!(health.max_partition_lag_bytes, Some(0));
    assert!(health.global_visibility_blocker_codes.is_empty());
    assert_eq!(
        health.global_durable_low_watermark_partition_ids,
        vec![0, 1]
    );
    assert_eq!(
        health.global_applied_low_watermark_partition_ids,
        vec![0, 1]
    );
    assert!(health.lagging_partition_ids.is_empty());
    assert!(health.straggler_partition_ids.is_empty());
    assert!(health.blocking_partition_ids.is_empty());
    assert!(health.missing_partitions.is_empty());
    assert!(health.visibility_actions.is_empty());
}

#[test]
fn partition_scale_health_names_lagging_blockers_and_skew() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        3,
        vec![
            partition_checkpoint(0, "0/16B9000", "0/16B9000"),
            partition_checkpoint(1, "0/16B9000", "0/16B7000"),
            partition_checkpoint(2, "0/16B8800", "0/16B8800"),
        ],
    )
    .expect("watermark summary");

    let health = PartitionScaleHealthSummary::from_watermark_summary(&summary);

    assert_eq!(health.status, PartitionScaleHealthStatus::LaggingPartitions);
    assert!(health.global_watermark_available);
    assert!(!health.global_visibility_releasable);
    assert_eq!(health.global_durable_lsn, Some("0/16B8800".to_string()));
    assert_eq!(health.global_applied_lsn, Some("0/16B7000".to_string()));
    assert_eq!(
        health.max_observed_durable_lsn,
        Some("0/16B9000".to_string())
    );
    assert_eq!(
        health.max_observed_applied_lsn,
        Some("0/16B9000".to_string())
    );
    assert_eq!(health.observed_durable_skew_bytes, Some(2048));
    assert_eq!(health.observed_applied_skew_bytes, Some(8192));
    assert_eq!(health.max_partition_lag_bytes, Some(8192));
    assert_eq!(
        health.global_visibility_blocker_codes,
        vec![
            "global_applied_low_watermark_blocked".to_string(),
            "partition_durable_apply_lag".to_string(),
        ]
    );
    assert_eq!(health.global_durable_low_watermark_partition_ids, vec![2]);
    assert_eq!(health.global_applied_low_watermark_partition_ids, vec![1]);
    assert_eq!(health.lagging_partition_ids, vec![1]);
    assert_eq!(health.straggler_partition_ids, vec![1, 2]);
    assert_eq!(health.blocking_partition_ids, vec![1]);
    assert!(health.missing_partitions.is_empty());
    assert_eq!(health.visibility_actions.len(), 2);
    assert_eq!(
        health.visibility_actions[0].code,
        "advance_global_applied_low_watermark"
    );
    assert_eq!(health.visibility_actions[0].partition_ids, vec![1]);
    assert_eq!(
        health.visibility_actions[0].command,
        "trellara apply --config <flow>"
    );
    assert_eq!(
        health.visibility_actions[1].code,
        "drain_lagging_partition_lanes"
    );
    assert_eq!(health.visibility_actions[1].partition_ids, vec![1, 2]);
}

#[test]
fn partition_scale_health_reports_uniform_lag_without_global_blocker() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        2,
        vec![
            partition_checkpoint(0, "0/16B9000", "0/16B8000"),
            partition_checkpoint(1, "0/16B9000", "0/16B8000"),
        ],
    )
    .expect("watermark summary");

    let health = PartitionScaleHealthSummary::from_watermark_summary(&summary);

    assert_eq!(health.status, PartitionScaleHealthStatus::LaggingPartitions);
    assert!(health.global_watermark_available);
    assert!(!health.global_visibility_releasable);
    assert_eq!(health.global_durable_lsn, Some("0/16B9000".to_string()));
    assert_eq!(health.global_applied_lsn, Some("0/16B8000".to_string()));
    assert_eq!(health.observed_applied_skew_bytes, Some(0));
    assert_eq!(health.max_partition_lag_bytes, Some(4096));
    assert_eq!(
        health.global_visibility_blocker_codes,
        vec!["partition_durable_apply_lag".to_string()]
    );
    assert_eq!(
        health.global_durable_low_watermark_partition_ids,
        vec![0, 1]
    );
    assert_eq!(
        health.global_applied_low_watermark_partition_ids,
        vec![0, 1]
    );
    assert_eq!(health.lagging_partition_ids, vec![0, 1]);
    assert!(health.straggler_partition_ids.is_empty());
    assert!(health.blocking_partition_ids.is_empty());
    assert!(health.missing_partitions.is_empty());
    assert_eq!(health.visibility_actions.len(), 1);
    assert_eq!(
        health.visibility_actions[0].code,
        "drain_lagging_partition_lanes"
    );
    assert_eq!(health.visibility_actions[0].partition_ids, vec![0, 1]);
}

#[test]
fn partition_scale_health_withholds_global_watermark_when_partitions_are_missing() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        4,
        vec![
            partition_checkpoint(0, "0/16B9000", "0/16B9000"),
            partition_checkpoint(2, "0/16B8800", "0/16B8800"),
        ],
    )
    .expect("watermark summary");

    let health = PartitionScaleHealthSummary::from_watermark_summary(&summary);

    assert_eq!(health.status, PartitionScaleHealthStatus::MissingPartitions);
    assert!(!health.global_watermark_available);
    assert!(!health.global_visibility_releasable);
    assert_eq!(health.global_durable_lsn, None);
    assert_eq!(health.global_applied_lsn, None);
    assert_eq!(
        health.max_observed_durable_lsn,
        Some("0/16B9000".to_string())
    );
    assert_eq!(
        health.max_observed_applied_lsn,
        Some("0/16B9000".to_string())
    );
    assert_eq!(health.observed_durable_skew_bytes, Some(2048));
    assert_eq!(health.observed_applied_skew_bytes, Some(2048));
    assert_eq!(health.max_partition_lag_bytes, Some(0));
    assert_eq!(
        health.global_visibility_blocker_codes,
        vec!["missing_partition_watermarks".to_string()]
    );
    assert!(health.global_durable_low_watermark_partition_ids.is_empty());
    assert!(health.global_applied_low_watermark_partition_ids.is_empty());
    assert!(health.lagging_partition_ids.is_empty());
    assert_eq!(health.straggler_partition_ids, vec![2]);
    assert!(health.blocking_partition_ids.is_empty());
    assert_eq!(health.missing_partitions, vec![1, 3]);
    assert_eq!(health.visibility_actions.len(), 1);
    assert_eq!(
        health.visibility_actions[0].code,
        "collect_missing_partition_watermarks"
    );
    assert_eq!(health.visibility_actions[0].partition_ids, vec![1, 3]);
    assert!(health.visibility_actions[0]
        .reason
        .contains("global visibility is withheld"));
}

#[test]
fn partition_scale_health_blocks_global_visibility_on_straggler_without_local_lag() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        2,
        vec![
            partition_checkpoint(0, "0/16B9000", "0/16B9000"),
            partition_checkpoint(1, "0/16B8000", "0/16B8000"),
        ],
    )
    .expect("watermark summary");

    let health = PartitionScaleHealthSummary::from_watermark_summary(&summary);

    assert_eq!(health.status, PartitionScaleHealthStatus::LaggingPartitions);
    assert!(health.global_watermark_available);
    assert!(!health.global_visibility_releasable);
    assert_eq!(health.global_applied_lsn, Some("0/16B8000".to_string()));
    assert!(health.lagging_partition_ids.is_empty());
    assert_eq!(health.blocking_partition_ids, vec![1]);
    assert_eq!(health.straggler_partition_ids, vec![1]);
    assert_eq!(
        health.global_visibility_blocker_codes,
        vec!["global_applied_low_watermark_blocked".to_string()]
    );
    assert_eq!(health.visibility_actions.len(), 1);
    assert_eq!(
        health.visibility_actions[0].code,
        "advance_global_applied_low_watermark"
    );
}
