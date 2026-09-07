use super::*;

#[test]
fn raw_cdc_writer_plan_marks_missing_required_sources() {
    let envelope = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )],
    );
    let writer_config = raw_writer_config(1)
        .with_required_sources(["store-001", "store-002"])
        .with_straggler_policy(LakeStragglerPolicy::PublishWithGaps { grace_ms: 30_000 });

    let plan =
        plan_raw_cdc_epoch_writes(&writer_config, &config(), &[envelope]).expect("writer plan");

    assert_eq!(
        plan.epoch_metadata.epoch_row.state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(plan.epoch_metadata.epoch_row.policy, "publish_with_gaps");
    assert_eq!(plan.epoch_metadata.epoch_row.required_source_count, 2);
    assert_eq!(plan.epoch_metadata.epoch_row.complete_source_count, 1);
    assert_eq!(plan.epoch_metadata.epoch_row.missing_source_count, 1);
    assert_eq!(
        plan.epoch_metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Match
    );
    let missing = plan
        .epoch_metadata
        .source_rows
        .iter()
        .find(|source| source.source_id == "store-002")
        .expect("missing source row");
    assert_eq!(missing.state, LakeEpochSourceState::Missing);
    assert_eq!(missing.transaction_count, 0);
    assert!(missing
        .lag_reason
        .as_deref()
        .unwrap_or_default()
        .contains("published gap epoch"));
}
