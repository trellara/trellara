use super::*;

#[test]
fn snapshot_table_progress_update_allows_forward_and_idempotent_evidence() {
    let copying =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, Some("0/16B8000"));
    let complete =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"));

    validate_snapshot_table_progress_update(&copying, &complete).expect("forward progress");
    validate_snapshot_table_progress_update(&complete, &complete).expect("idempotent replay");
}

#[test]
fn snapshot_table_progress_record_requires_identity_and_nonnegative_rows() {
    validate_snapshot_table_progress_record(&snapshot_table_progress_for_test(
        SnapshotRunState::CopyingTable,
        12,
        None,
    ))
    .expect("active copy can start before watermark is known");

    let mut missing_relation =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, None);
    missing_relation.relation = " ".to_string();
    let error = validate_snapshot_table_progress_record(&missing_relation)
        .expect_err("empty relation rejected");
    assert!(error.to_string().contains("relation must not be empty"));

    let mut missing_run_id =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, None);
    missing_run_id.run_id = "\t".to_string();
    let error =
        validate_snapshot_table_progress_record(&missing_run_id).expect_err("empty run rejected");
    assert!(error.to_string().contains("run_id must not be empty"));

    let error = validate_snapshot_table_progress_record(&snapshot_table_progress_for_test(
        SnapshotRunState::CopyingTable,
        -1,
        None,
    ))
    .expect_err("negative rows rejected");
    assert!(error.to_string().contains("negative copied rows"));
}

#[test]
fn snapshot_table_progress_record_rejects_padded_identity_fields() {
    for (field, expected) in [
        (
            "source_id",
            "source_id must not contain surrounding whitespace",
        ),
        (
            "dataset_id",
            "dataset_id must not contain surrounding whitespace",
        ),
        ("run_id", "run_id must not contain surrounding whitespace"),
        (
            "relation",
            "relation must not contain surrounding whitespace",
        ),
    ] {
        let mut progress =
            snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, None);
        match field {
            "source_id" => progress.source_id = " source".to_string(),
            "dataset_id" => progress.dataset_id = "dataset ".to_string(),
            "run_id" => progress.run_id = "\trun".to_string(),
            "relation" => progress.relation = " public.sales".to_string(),
            _ => unreachable!("covered snapshot table identity field"),
        }

        let error = validate_snapshot_table_progress_record(&progress)
            .expect_err("padded identity rejected");
        assert!(error.to_string().contains(expected));
    }
}

#[test]
fn snapshot_table_progress_record_requires_watermark_for_terminal_evidence() {
    for state in [
        SnapshotRunState::CopyComplete,
        SnapshotRunState::StreamHandoffReady,
        SnapshotRunState::Streaming,
        SnapshotRunState::Verified,
        SnapshotRunState::FailedRecoverable,
    ] {
        let error = validate_snapshot_table_progress_record(&snapshot_table_progress_for_test(
            state, 40, None,
        ))
        .expect_err("missing watermark rejected");
        assert!(error
            .to_string()
            .contains(&format!("cannot enter {state} without a watermark LSN")));
    }
}

#[test]
fn snapshot_table_progress_record_rejects_malformed_or_zero_watermark_lsn() {
    for watermark_lsn in ["bad-lsn", "0/0"] {
        let error = validate_snapshot_table_progress_record(&snapshot_table_progress_for_test(
            SnapshotRunState::CopyComplete,
            40,
            Some(watermark_lsn),
        ))
        .expect_err("invalid watermark rejected");

        assert!(error.to_string().contains("watermark_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[test]
fn snapshot_table_progress_update_rejects_backward_state() {
    let complete =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"));
    let copying =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 40, Some("0/16B8000"));

    let error = validate_snapshot_table_progress_update(&complete, &copying)
        .expect_err("backward state rejected");
    assert!(error
        .to_string()
        .contains("invalid snapshot run transition from copy_complete to copying_table"));
}

#[test]
fn snapshot_table_progress_update_rejects_lower_copied_rows() {
    let current =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 40, Some("0/16B8000"));
    let next =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, Some("0/16B8000"));

    let error = validate_snapshot_table_progress_update(&current, &next)
        .expect_err("lower copied rows rejected");
    assert!(error
        .to_string()
        .contains("cannot move copied rows backward"));
}

#[test]
fn snapshot_table_progress_update_rejects_watermark_rewrite() {
    let current =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, Some("0/16B8000"));
    let next =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B9000"));

    let error = validate_snapshot_table_progress_update(&current, &next)
        .expect_err("changed watermark rejected");
    assert!(error.to_string().contains("cannot change watermark"));
}

#[test]
fn snapshot_table_progress_update_rejects_watermark_clear() {
    let current =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, Some("0/16B8000"));
    let next = snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, None);

    let error = validate_snapshot_table_progress_update(&current, &next)
        .expect_err("cleared watermark rejected");
    assert!(error.to_string().contains("cannot clear watermark"));
}

#[test]
fn snapshot_table_progress_update_rejects_identity_rewrites() {
    let current =
        snapshot_table_progress_for_test(SnapshotRunState::CopyingTable, 12, Some("0/16B8000"));

    let mut wrong_source =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"));
    wrong_source.source_id = "source-b".to_string();
    let error = validate_snapshot_table_progress_update(&current, &wrong_source)
        .expect_err("source rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change source_id from source to source-b"));

    let mut wrong_dataset =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"));
    wrong_dataset.dataset_id = "orders".to_string();
    let error = validate_snapshot_table_progress_update(&current, &wrong_dataset)
        .expect_err("dataset rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change dataset_id from dataset to orders"));

    let mut wrong_run =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"));
    wrong_run.run_id = "snapshot-run-2".to_string();
    let error = validate_snapshot_table_progress_update(&current, &wrong_run)
        .expect_err("run rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change run_id from run to snapshot-run-2"));

    let mut wrong_relation =
        snapshot_table_progress_for_test(SnapshotRunState::CopyComplete, 40, Some("0/16B8000"));
    wrong_relation.relation = "public.orders".to_string();
    let error = validate_snapshot_table_progress_update(&current, &wrong_relation)
        .expect_err("relation rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change relation from public.sales to public.orders"));
}
