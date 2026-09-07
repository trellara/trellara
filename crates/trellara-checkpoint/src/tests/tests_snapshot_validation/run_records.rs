use super::*;

#[test]
fn snapshot_run_record_requires_flow_and_run_identity() {
    let mut run = snapshot_run_for_test(SnapshotRunState::Planned, None, 0);

    run.source_id = "  ".to_string();
    let error = validate_snapshot_run_record(&run).expect_err("empty source rejected");
    assert!(error.to_string().contains("source_id must not be empty"));

    run.source_id = "source".to_string();
    run.dataset_id.clear();
    let error = validate_snapshot_run_record(&run).expect_err("empty dataset rejected");
    assert!(error.to_string().contains("dataset_id must not be empty"));

    run.dataset_id = "dataset".to_string();
    run.run_id = "\t".to_string();
    let error = validate_snapshot_run_record(&run).expect_err("empty run id rejected");
    assert!(error.to_string().contains("run_id must not be empty"));
}

#[test]
fn snapshot_run_record_rejects_padded_identity_fields() {
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
    ] {
        let mut run = snapshot_run_for_test(SnapshotRunState::Planned, None, 0);
        match field {
            "source_id" => run.source_id = " source".to_string(),
            "dataset_id" => run.dataset_id = "dataset ".to_string(),
            "run_id" => run.run_id = "\trun".to_string(),
            _ => unreachable!("covered snapshot run identity field"),
        }

        let error = validate_snapshot_run_record(&run).expect_err("padded identity rejected");
        assert!(error.to_string().contains(expected));
    }
}

#[test]
fn snapshot_run_record_requires_slot_name_after_slot_creation() {
    for state in [
        SnapshotRunState::SlotCreated,
        SnapshotRunState::SnapshotExported,
        SnapshotRunState::CopyingTable,
        SnapshotRunState::CopyComplete,
        SnapshotRunState::StreamHandoffReady,
        SnapshotRunState::Streaming,
        SnapshotRunState::Verified,
        SnapshotRunState::FailedRecoverable,
    ] {
        let consistent_lsn = if state == SnapshotRunState::SlotCreated {
            None
        } else {
            Some("0/16B8000")
        };
        let mut run = snapshot_run_for_test(state, consistent_lsn, 10);
        if state == SnapshotRunState::FailedRecoverable {
            run.failure_reason = Some("target connection dropped".to_string());
        }
        run.slot_name = " ".to_string();

        let error = validate_snapshot_run_record(&run).expect_err("slot name rejected");
        assert!(error.to_string().contains("without a slot name"));
    }
}

#[test]
fn snapshot_run_record_requires_consistent_lsn_after_export_boundary() {
    validate_snapshot_run_record(&snapshot_run_for_test(
        SnapshotRunState::SlotCreated,
        None,
        0,
    ))
    .expect("slot creation may not have exported snapshot evidence yet");

    for state in [
        SnapshotRunState::SnapshotExported,
        SnapshotRunState::CopyingTable,
        SnapshotRunState::CopyComplete,
        SnapshotRunState::StreamHandoffReady,
        SnapshotRunState::Streaming,
        SnapshotRunState::Verified,
        SnapshotRunState::FailedRecoverable,
    ] {
        let mut run = snapshot_run_for_test(state, None, 10);
        if state == SnapshotRunState::FailedRecoverable {
            run.failure_reason = Some("target connection dropped".to_string());
        }
        let error = validate_snapshot_run_record(&run)
            .expect_err("post-export states require consistent LSN");
        assert!(error.to_string().contains("without a consistent LSN"));
    }

    validate_snapshot_run_record(&snapshot_run_for_test(
        SnapshotRunState::StreamHandoffReady,
        Some("0/16B8000"),
        40,
    ))
    .expect("handoff-ready run has boundary evidence");
}

#[test]
fn snapshot_run_record_requires_actionable_failure_reason_for_recoverable_failure() {
    let error = validate_snapshot_run_record(&snapshot_run_for_test(
        SnapshotRunState::FailedRecoverable,
        Some("0/16B8000"),
        10,
    ))
    .expect_err("failed recoverable run needs reason");
    assert!(error
        .to_string()
        .contains("cannot enter failed_recoverable without a failure reason"));

    let mut empty_reason =
        snapshot_run_for_test(SnapshotRunState::FailedRecoverable, Some("0/16B8000"), 10);
    empty_reason.failure_reason = Some(" ".to_string());
    let error = validate_snapshot_run_record(&empty_reason).expect_err("empty reason rejected");
    assert!(error
        .to_string()
        .contains("failure_reason must not be empty"));

    let mut padded_reason =
        snapshot_run_for_test(SnapshotRunState::FailedRecoverable, Some("0/16B8000"), 10);
    padded_reason.failure_reason = Some(" target connection dropped ".to_string());
    let error = validate_snapshot_run_record(&padded_reason).expect_err("padded reason rejected");
    assert!(error
        .to_string()
        .contains("failure_reason must not contain surrounding whitespace"));

    let mut valid_reason =
        snapshot_run_for_test(SnapshotRunState::FailedRecoverable, Some("0/16B8000"), 10);
    valid_reason.failure_reason = Some("target connection dropped".to_string());
    validate_snapshot_run_record(&valid_reason).expect("actionable failure accepted");
}

#[test]
fn snapshot_run_record_rejects_malformed_or_zero_consistent_lsn() {
    for consistent_lsn in ["not-an-lsn", "0/0"] {
        let error = validate_snapshot_run_record(&snapshot_run_for_test(
            SnapshotRunState::StreamHandoffReady,
            Some(consistent_lsn),
            40,
        ))
        .expect_err("invalid consistent LSN rejected");

        assert!(error.to_string().contains("consistent_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[test]
fn snapshot_run_record_rejects_negative_copied_rows() {
    let error = validate_snapshot_run_record(&snapshot_run_for_test(
        SnapshotRunState::SlotCreated,
        None,
        -1,
    ))
    .expect_err("negative copied rows rejected");

    assert!(error.to_string().contains("negative copied rows"));
}

#[test]
fn snapshot_run_update_allows_forward_and_idempotent_evidence() {
    let exported = snapshot_run_for_test(SnapshotRunState::SnapshotExported, Some("0/16B8000"), 0);
    let copying = snapshot_run_for_test(SnapshotRunState::CopyingTable, Some("0/16B8000"), 12);

    validate_snapshot_run_update(&exported, &copying).expect("forward progress");
    validate_snapshot_run_update(&copying, &copying).expect("idempotent progress");
}

#[test]
fn snapshot_run_update_rejects_lower_copied_rows() {
    let current = snapshot_run_for_test(SnapshotRunState::CopyingTable, Some("0/16B8000"), 40);
    let next = snapshot_run_for_test(SnapshotRunState::CopyingTable, Some("0/16B8000"), 12);

    let error =
        validate_snapshot_run_update(&current, &next).expect_err("lower copied rows rejected");
    assert!(error
        .to_string()
        .contains("cannot move copied rows backward"));
}

#[test]
fn snapshot_run_update_rejects_consistent_lsn_rewrite_or_clear() {
    let current = snapshot_run_for_test(SnapshotRunState::CopyingTable, Some("0/16B8000"), 12);
    let changed = snapshot_run_for_test(SnapshotRunState::CopyComplete, Some("0/16B9000"), 40);

    let error = validate_snapshot_run_update(&current, &changed).expect_err("changed LSN rejected");
    assert!(error.to_string().contains("cannot change consistent LSN"));

    let cleared = snapshot_run_for_test(SnapshotRunState::CopyComplete, None, 40);
    let error = validate_snapshot_run_update(&current, &cleared).expect_err("cleared LSN rejected");
    assert!(error.to_string().contains("cannot clear consistent LSN"));
}

#[test]
fn snapshot_run_update_rejects_identity_rewrites() {
    let current = snapshot_run_for_test(SnapshotRunState::CopyingTable, Some("0/16B8000"), 12);

    let mut wrong_source =
        snapshot_run_for_test(SnapshotRunState::CopyComplete, Some("0/16B8000"), 40);
    wrong_source.source_id = "source-b".to_string();
    let error =
        validate_snapshot_run_update(&current, &wrong_source).expect_err("source rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change source_id from source to source-b"));

    let mut wrong_dataset =
        snapshot_run_for_test(SnapshotRunState::CopyComplete, Some("0/16B8000"), 40);
    wrong_dataset.dataset_id = "orders".to_string();
    let error = validate_snapshot_run_update(&current, &wrong_dataset)
        .expect_err("dataset rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change dataset_id from dataset to orders"));

    let mut wrong_run =
        snapshot_run_for_test(SnapshotRunState::CopyComplete, Some("0/16B8000"), 40);
    wrong_run.run_id = "snapshot-run-2".to_string();
    let error =
        validate_snapshot_run_update(&current, &wrong_run).expect_err("run rewrite rejected");
    assert!(error
        .to_string()
        .contains("cannot change run_id from run to snapshot-run-2"));
}
