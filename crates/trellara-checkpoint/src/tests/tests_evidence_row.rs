use super::*;

#[test]
fn reseed_event_from_parts_rejects_invalid_persisted_watermark() {
    let error = evidence_row::reseed_event_from_parts(
        "source".to_string(),
        "sales".to_string(),
        "not-a-lsn".to_string(),
        1,
        10,
        "2026-08-26T00:00:00Z".to_string(),
    )
    .expect_err("invalid reseed row");

    assert!(error.to_string().contains("watermark_lsn"));
}

#[test]
fn snapshot_handoff_event_from_parts_rejects_negative_copied_rows() {
    let error = evidence_row::snapshot_handoff_event_from_parts(
        "source".to_string(),
        "sales".to_string(),
        "public.sales".to_string(),
        "0/16B9000".to_string(),
        -1,
        "2026-08-26T00:00:00Z".to_string(),
    )
    .expect_err("invalid snapshot handoff row");

    assert!(error.to_string().contains("negative copied rows"));
}

#[test]
fn validation_event_from_parts_rejects_corrupted_drift_counts() {
    let error = evidence_row::validation_event_from_parts(evidence_row::ValidationEventParts {
        source_id: "source".to_string(),
        dataset_id: "sales".to_string(),
        source_watermark_lsn: "0/16B9000".to_string(),
        target_watermark_lsn: "0/16B8000".to_string(),
        converged: false,
        table_count: 2,
        drift_count: 2,
        drift_relations: vec!["public.sales".to_string()],
        evidence_sha256: Some("a".repeat(64)),
        completed_at: "2026-08-26T00:00:00Z".to_string(),
    })
    .expect_err("invalid validation row");

    assert!(error.to_string().contains("drift relation count"));
}
