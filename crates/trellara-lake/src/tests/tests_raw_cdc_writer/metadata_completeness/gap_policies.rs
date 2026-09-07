use super::*;

#[test]
fn metadata_plan_marks_missing_required_source_as_gap() {
    let mut input = input();
    input.required_sources = required_sources();
    input.straggler_policy = LakeStragglerPolicy::PublishWithGaps { grace_ms: 30_000 };
    input.source_rows = vec![source_row("store-001")];

    let metadata = build_epoch_metadata(input).expect("metadata plan");

    assert_eq!(
        metadata.epoch_row.state,
        LakeCompletenessState::CompleteWithGaps
    );
    assert_eq!(metadata.epoch_row.policy, "publish_with_gaps");
    assert_eq!(metadata.epoch_row.required_source_count, 2);
    assert_eq!(metadata.epoch_row.complete_source_count, 1);
    assert_eq!(metadata.epoch_row.missing_source_count, 1);
    assert_eq!(metadata.epoch_row.quarantined_source_count, 0);
    assert_eq!(
        metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Match
    );
    let missing = source(&metadata.source_rows, "store-002");
    assert_eq!(missing.state, LakeEpochSourceState::Missing);
    assert_eq!(missing.transaction_count, 0);
    assert!(missing
        .lag_reason
        .as_deref()
        .unwrap_or_default()
        .contains("published gap epoch"));
}

#[test]
fn metadata_plan_keeps_wait_all_required_gap_open() {
    let mut input = input();
    input.required_sources = required_sources();
    input.source_rows = vec![source_row("store-001")];

    let metadata = build_epoch_metadata(input).expect("metadata plan");

    assert_eq!(metadata.epoch_row.state, LakeCompletenessState::Open);
    assert_eq!(metadata.epoch_row.policy, "wait_all_required");
    assert_eq!(metadata.epoch_row.missing_source_count, 1);
    assert_eq!(
        metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Unknown
    );
    assert_eq!(
        source(&metadata.source_rows, "store-002").state,
        LakeEpochSourceState::Lagging
    );
}

#[test]
fn metadata_plan_does_not_count_non_required_observed_source_as_complete() {
    let mut input = input();
    input.required_sources = required_sources();
    input.source_rows = vec![source_row("store-003")];

    let metadata = build_epoch_metadata(input).expect("metadata plan");

    assert_eq!(metadata.epoch_row.required_source_count, 2);
    assert_eq!(metadata.epoch_row.complete_source_count, 0);
    assert_eq!(metadata.epoch_row.missing_source_count, 2);
    assert_eq!(
        metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Unknown
    );
    assert_eq!(
        source(&metadata.source_rows, "store-003").state,
        LakeEpochSourceState::Complete
    );
}

#[test]
fn metadata_plan_quarantines_required_source_gap() {
    let mut input = input();
    input.required_sources = required_sources();
    input.straggler_policy = LakeStragglerPolicy::QuarantineOnGap;
    input.source_rows = vec![source_row("store-001")];

    let metadata = build_epoch_metadata(input).expect("metadata plan");

    assert_eq!(metadata.epoch_row.state, LakeCompletenessState::Quarantined);
    assert_eq!(metadata.epoch_row.policy, "quarantine_on_gap");
    assert_eq!(metadata.epoch_row.missing_source_count, 0);
    assert_eq!(metadata.epoch_row.quarantined_source_count, 1);
    assert_eq!(
        metadata.verification_row.checksum_status,
        LakeEpochVerificationStatus::Unknown
    );
    assert_eq!(
        source(&metadata.source_rows, "store-002").state,
        LakeEpochSourceState::Quarantined
    );
    assert_eq!(metadata.quarantine_rows.len(), 1);
    let quarantine = &metadata.quarantine_rows[0];
    assert_eq!(quarantine.epoch_id, "epoch-1");
    assert_eq!(quarantine.source_id.as_deref(), Some("store-002"));
    assert_eq!(quarantine.transaction_id, None);
    assert_eq!(quarantine.commit_lsn, None);
    assert_eq!(quarantine.reason, "source_gap_quarantined");
    assert_eq!(
        quarantine.details.as_deref(),
        Some("required source missing under quarantine-on-gap policy")
    );
    assert!(quarantine
        .recovery_command
        .as_deref()
        .unwrap_or_default()
        .contains("trellara lake fanin verify"));
}

#[test]
fn metadata_plan_records_observed_quarantined_source_row() {
    let mut input = input();
    let mut quarantined = source_row("store-001");
    quarantined.state = LakeEpochSourceState::Quarantined;
    quarantined.lag_reason = Some("conflicting duplicate source evidence".to_string());
    input.source_rows = vec![quarantined];

    let metadata = build_epoch_metadata(input).expect("metadata plan");

    assert_eq!(metadata.epoch_row.state, LakeCompletenessState::Quarantined);
    assert_eq!(metadata.epoch_row.quarantined_source_count, 1);
    assert_eq!(metadata.quarantine_rows.len(), 1);
    let quarantine = &metadata.quarantine_rows[0];
    assert_eq!(quarantine.source_id.as_deref(), Some("store-001"));
    assert_eq!(quarantine.commit_lsn.as_deref(), Some("0/16B0100"));
    assert_eq!(quarantine.reason, "source_evidence_quarantined");
    assert_eq!(
        quarantine.details.as_deref(),
        Some("conflicting duplicate source evidence")
    );
}
