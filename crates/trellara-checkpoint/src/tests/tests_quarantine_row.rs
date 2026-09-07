use super::*;

#[test]
fn quarantine_from_parts_accepts_valid_persisted_row() {
    let record = quarantine_row::quarantine_from_parts(valid_quarantine_parts())
        .expect("valid quarantine row");

    assert_eq!(record.transaction_id, "tx-1");
    assert_eq!(record.attempt_count, 1);
}

#[test]
fn quarantine_from_parts_rejects_invalid_commit_lsn() {
    let mut parts = valid_quarantine_parts();
    parts.commit_lsn = "0/0".to_string();

    let error = quarantine_row::quarantine_from_parts(parts).expect_err("invalid quarantine row");

    assert!(error.to_string().contains("commit_lsn"));
}

#[test]
fn quarantine_from_parts_rejects_empty_reason() {
    let mut parts = valid_quarantine_parts();
    parts.reason.clear();

    let error = quarantine_row::quarantine_from_parts(parts).expect_err("empty reason");

    assert!(error.to_string().contains("reason must not be empty"));
}

fn valid_quarantine_parts() -> quarantine_row::QuarantineRowParts {
    quarantine_row::QuarantineRowParts {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        commit_lsn: "0/16B9000".to_string(),
        reason: "target_postgres_error".to_string(),
        detail: "duplicate key".to_string(),
        attempt_count: 1,
        last_seen_at: "2026-08-26T00:00:00Z".to_string(),
    }
}
