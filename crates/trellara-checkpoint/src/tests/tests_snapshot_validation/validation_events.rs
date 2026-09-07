use super::*;

#[test]
fn validation_event_accepts_consistent_convergence_evidence() {
    validate_validation_event(&validation_event_for_test(
        true,
        "0/16B9000",
        "0/16B9000",
        0,
    ))
    .expect("converged validation evidence");

    validate_validation_event(&validation_event_for_test(
        false,
        "0/16B9000",
        "0/16B8000",
        0,
    ))
    .expect("non-converged watermark lag evidence");

    validate_validation_event(&validation_event_for_test(
        false,
        "0/16B9000",
        "0/16B9000",
        2,
    ))
    .expect("non-converged drift evidence");
}

#[test]
fn validation_event_rejects_missing_watermark_or_identity() {
    let error = validate_validation_event(&validation_event_for_test(true, "", "0/16B9000", 0))
        .expect_err("missing source watermark rejected");
    assert!(error.to_string().contains("source watermark LSN"));

    let mut event = validation_event_for_test(true, "0/16B9000", "0/16B9000", 0);
    event.dataset_id = " ".to_string();
    let error = validate_validation_event(&event).expect_err("empty dataset rejected");
    assert!(error.to_string().contains("dataset_id must not be empty"));
}

#[test]
fn validation_event_rejects_malformed_or_zero_watermark_lsn() {
    for watermark_lsn in ["bad-lsn", "0/0"] {
        let error = validate_validation_event(&validation_event_for_test(
            false,
            watermark_lsn,
            "0/16B9000",
            1,
        ))
        .expect_err("invalid source watermark rejected");

        assert!(error.to_string().contains("source_watermark_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));

        let error = validate_validation_event(&validation_event_for_test(
            false,
            "0/16B9000",
            watermark_lsn,
            1,
        ))
        .expect_err("invalid target watermark rejected");

        assert!(error.to_string().contains("target_watermark_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[test]
fn validation_event_rejects_impossible_counts() {
    let mut event = validation_event_for_test(false, "0/16B9000", "0/16B8000", 1);
    event.table_count = 0;
    let error = validate_validation_event(&event).expect_err("zero tables rejected");
    assert!(error.to_string().contains("at least one table"));

    let mut event = validation_event_for_test(false, "0/16B9000", "0/16B8000", 1);
    event.drift_count = 4;
    let error = validate_validation_event(&event).expect_err("drift exceeds tables rejected");
    assert!(error.to_string().contains("cannot exceed table count"));
}

#[test]
fn validation_event_rejects_inconsistent_drift_metadata() {
    let mut event = validation_event_for_test(false, "0/16B9000", "0/16B8000", 2);
    event.drift_relations.pop();
    let error = validate_validation_event(&event).expect_err("drift relation mismatch rejected");
    assert!(error.to_string().contains("does not match drift count"));

    let mut event = validation_event_for_test(false, "0/16B9000", "0/16B8000", 1);
    event.drift_relations = vec![" ".to_string()];
    let error = validate_validation_event(&event).expect_err("empty drift relation rejected");
    assert!(error
        .to_string()
        .contains("drift relations must not be empty"));
}

#[test]
fn validation_event_rejects_converged_drift_or_watermark_mismatch() {
    let error = validate_validation_event(&validation_event_for_test(
        true,
        "0/16B9000",
        "0/16B9000",
        1,
    ))
    .expect_err("converged drift rejected");
    assert!(error.to_string().contains("cannot include drift"));

    let error = validate_validation_event(&validation_event_for_test(
        true,
        "0/16B9000",
        "0/16B8000",
        0,
    ))
    .expect_err("converged watermark mismatch rejected");
    assert!(error.to_string().contains("requires matching watermarks"));
}

#[test]
fn validation_event_rejects_non_converged_without_failure_evidence() {
    let error = validate_validation_event(&validation_event_for_test(
        false,
        "0/16B9000",
        "0/16B9000",
        0,
    ))
    .expect_err("non-converged event without evidence rejected");

    assert!(error
        .to_string()
        .contains("must include drift or mismatched watermarks"));
}

#[test]
fn validation_event_rejects_malformed_evidence_digest() {
    for digest in [
        "short",
        " aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    ] {
        let mut event = validation_event_for_test(true, "0/16B9000", "0/16B9000", 0);
        event.evidence_sha256 = Some(digest.to_string());

        let error = validate_validation_event(&event).expect_err("bad digest rejected");

        assert!(error.to_string().contains("evidence_sha256"));
    }
}
