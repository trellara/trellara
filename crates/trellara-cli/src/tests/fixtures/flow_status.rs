use super::*;

pub(in crate::tests) fn clean_status() -> FlowStatusSummary {
    FlowStatusSummary::new(clean_status_parts())
}

pub(in crate::tests) fn clean_status_parts() -> FlowStatusParts {
    FlowStatusParts {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        source_slot: healthy_slot(),
        subscription_conflicts: Vec::new(),
        source_wal_retention_warn_bytes: Some(1_000_000),
        source_stream_spill_threshold_changes:
            trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        source_stream_spill_dir: None,
        source: Some(caught_up_lag()),
        target: Some(caught_up_lag()),
        partition_watermarks: None,
        source_schema_drift: None,
        latest_quarantine: None,
        latest_reseed: Some(latest_reseed()),
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(latest_snapshot_run()),
        latest_validation: Some(latest_validation(true)),
        recovery_actions: Vec::new(),
    }
}
