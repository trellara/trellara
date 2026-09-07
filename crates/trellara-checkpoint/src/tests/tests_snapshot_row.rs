use super::*;

#[test]
fn snapshot_run_from_parts_accepts_valid_persisted_row() {
    let run =
        snapshot_row::snapshot_run_from_parts(snapshot_run_parts(SnapshotRunState::CopyComplete))
            .expect("valid snapshot run row");

    assert_eq!(run.run_id, "snapshot-1");
    assert_eq!(run.consistent_lsn.as_deref(), Some("0/16B9000"));
}

#[test]
fn snapshot_run_from_parts_rejects_missing_required_lsn() {
    let mut parts = snapshot_run_parts(SnapshotRunState::CopyComplete);
    parts.consistent_lsn = None;

    let error = snapshot_row::snapshot_run_from_parts(parts).expect_err("invalid snapshot row");

    assert!(error.to_string().contains("consistent LSN"));
}

#[test]
fn snapshot_table_progress_from_parts_rejects_missing_watermark() {
    let mut parts = table_progress_parts(SnapshotRunState::CopyComplete);
    parts.watermark_lsn = None;

    let error =
        snapshot_row::snapshot_table_progress_from_parts(parts).expect_err("invalid table row");

    assert!(error.to_string().contains("without a watermark LSN"));
}

fn snapshot_run_parts(state: SnapshotRunState) -> snapshot_row::SnapshotRunParts {
    snapshot_row::SnapshotRunParts {
        source_id: "source".to_string(),
        dataset_id: "sales".to_string(),
        run_id: "snapshot-1".to_string(),
        state,
        slot_name: "trellara_slot".to_string(),
        consistent_lsn: Some("0/16B9000".to_string()),
        current_relation: Some("public.sales".to_string()),
        copied_rows: 10,
        failure_reason: None,
        started_at: "2026-08-26T00:00:00Z".to_string(),
        updated_at: "2026-08-26T00:00:01Z".to_string(),
    }
}

fn table_progress_parts(state: SnapshotRunState) -> snapshot_row::SnapshotTableProgressParts {
    snapshot_row::SnapshotTableProgressParts {
        source_id: "source".to_string(),
        dataset_id: "sales".to_string(),
        run_id: "snapshot-1".to_string(),
        relation: "public.sales".to_string(),
        state,
        copied_rows: 10,
        watermark_lsn: Some("0/16B9000".to_string()),
        updated_at: "2026-08-26T00:00:01Z".to_string(),
    }
}
