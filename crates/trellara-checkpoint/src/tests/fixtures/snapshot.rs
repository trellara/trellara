use super::*;

pub(in crate::tests) fn snapshot_run_for_test(
    state: SnapshotRunState,
    consistent_lsn: Option<&str>,
    copied_rows: i64,
) -> SnapshotRun {
    SnapshotRun {
        source_id: "source".to_string(),
        dataset_id: "dataset".to_string(),
        run_id: "run".to_string(),
        state,
        slot_name: "trellara_snapshot_slot".to_string(),
        consistent_lsn: consistent_lsn.map(str::to_string),
        current_relation: None,
        copied_rows,
        failure_reason: None,
        started_at: String::new(),
        updated_at: String::new(),
    }
}

pub(in crate::tests) fn reseed_event_for_test(
    watermark_lsn: &str,
    table_count: i64,
    copied_rows: i64,
) -> ReseedEvent {
    ReseedEvent {
        source_id: "source".to_string(),
        dataset_id: "dataset".to_string(),
        watermark_lsn: watermark_lsn.to_string(),
        table_count,
        copied_rows,
        completed_at: String::new(),
    }
}

pub(in crate::tests) fn snapshot_handoff_event_for_test(
    watermark_lsn: &str,
    copied_rows: i64,
) -> SnapshotHandoffEvent {
    SnapshotHandoffEvent {
        source_id: "source".to_string(),
        dataset_id: "dataset".to_string(),
        relation: "public.sales".to_string(),
        watermark_lsn: watermark_lsn.to_string(),
        copied_rows,
        completed_at: String::new(),
    }
}

pub(in crate::tests) fn validation_event_for_test(
    converged: bool,
    source_watermark_lsn: &str,
    target_watermark_lsn: &str,
    drift_count: i64,
) -> ValidationEvent {
    let drift_relations = (0..drift_count)
        .map(|index| format!("public.table_{index}"))
        .collect();
    ValidationEvent {
        source_id: "source".to_string(),
        dataset_id: "dataset".to_string(),
        source_watermark_lsn: source_watermark_lsn.to_string(),
        target_watermark_lsn: target_watermark_lsn.to_string(),
        converged,
        table_count: 3,
        drift_count,
        drift_relations,
        evidence_sha256: Some("a".repeat(64)),
        completed_at: String::new(),
    }
}

pub(in crate::tests) fn snapshot_table_progress_for_test(
    state: SnapshotRunState,
    copied_rows: i64,
    watermark_lsn: Option<&str>,
) -> SnapshotTableProgress {
    SnapshotTableProgress {
        source_id: "source".to_string(),
        dataset_id: "dataset".to_string(),
        run_id: "run".to_string(),
        relation: "public.sales".to_string(),
        state,
        copied_rows,
        watermark_lsn: watermark_lsn.map(str::to_string),
        updated_at: String::new(),
    }
}
