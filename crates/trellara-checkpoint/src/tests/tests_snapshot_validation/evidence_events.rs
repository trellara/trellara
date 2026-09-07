use super::*;

#[test]
fn reseed_event_requires_watermark_and_flow_identity() {
    validate_reseed_event(&reseed_event_for_test("0/16B8000", 1, 40))
        .expect("complete reseed evidence");

    let error = validate_reseed_event(&reseed_event_for_test("", 1, 40))
        .expect_err("missing watermark rejected");
    assert!(error.to_string().contains("without a watermark LSN"));

    let mut event = reseed_event_for_test("0/16B8000", 1, 40);
    event.source_id = " ".to_string();
    let error = validate_reseed_event(&event).expect_err("empty source rejected");
    assert!(error.to_string().contains("source_id must not be empty"));
}

#[test]
fn reseed_event_rejects_impossible_copy_counts() {
    let error = validate_reseed_event(&reseed_event_for_test("0/16B8000", 0, 40))
        .expect_err("zero table reseed rejected");
    assert!(error.to_string().contains("at least one table"));

    let error = validate_reseed_event(&reseed_event_for_test("0/16B8000", 1, -1))
        .expect_err("negative copied rows rejected");
    assert!(error.to_string().contains("negative copied rows"));
}

#[test]
fn reseed_event_rejects_malformed_or_zero_watermark_lsn() {
    for watermark_lsn in ["bad-lsn", "0/0"] {
        let error = validate_reseed_event(&reseed_event_for_test(watermark_lsn, 1, 40))
            .expect_err("invalid reseed watermark rejected");

        assert!(error.to_string().contains("watermark_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}

#[test]
fn snapshot_handoff_event_requires_boundary_evidence() {
    validate_snapshot_handoff_event(&snapshot_handoff_event_for_test("0/16B8000", 40))
        .expect("complete handoff evidence");

    let mut missing_watermark = snapshot_handoff_event_for_test("", 40);
    let error = validate_snapshot_handoff_event(&missing_watermark)
        .expect_err("missing watermark rejected");
    assert!(error.to_string().contains("without a watermark LSN"));

    missing_watermark.watermark_lsn = "0/16B8000".to_string();
    missing_watermark.relation = "  ".to_string();
    let error =
        validate_snapshot_handoff_event(&missing_watermark).expect_err("empty relation rejected");
    assert!(error.to_string().contains("relation must not be empty"));
}

#[test]
fn snapshot_handoff_event_rejects_negative_copied_rows() {
    let error = validate_snapshot_handoff_event(&snapshot_handoff_event_for_test("0/16B8000", -1))
        .expect_err("negative copied rows rejected");

    assert!(error.to_string().contains("negative copied rows"));
}

#[test]
fn snapshot_handoff_event_rejects_malformed_or_zero_watermark_lsn() {
    for watermark_lsn in ["bad-lsn", "0/0"] {
        let error =
            validate_snapshot_handoff_event(&snapshot_handoff_event_for_test(watermark_lsn, 40))
                .expect_err("invalid snapshot handoff watermark rejected");

        assert!(error.to_string().contains("watermark_lsn"));
        assert!(error.to_string().contains("non-zero PostgreSQL LSN"));
    }
}
