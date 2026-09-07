use super::*;

#[test]
fn epoch_summary_marks_all_required_sources_complete() {
    let first = envelope_for(
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
    let second = envelope_for(
        "store-002",
        "tx-2",
        "0/16B6C70",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-2", "20", "2026-01-01")),
        )],
    );

    let summary = build_epoch_summary(
        &epoch_config(
            &["store-001", "store-002"],
            LakeStragglerPolicy::WaitAllRequired,
        ),
        &[first.clone(), second.clone()],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::Complete);
    assert_eq!(summary.required_source_count, 2);
    assert_eq!(summary.complete_source_count, 2);
    assert_eq!(summary.transaction_count, 2);
    assert_eq!(summary.change_count, 2);
    assert_eq!(summary.checksum_rollup, first.checksum ^ second.checksum);
    assert_eq!(
        summary.verification.status,
        LakeEpochVerificationStatus::Match
    );
    assert_eq!(summary.tables[0].relation, "public.sales");
    assert_eq!(summary.tables[0].transaction_count, 2);
}

#[test]
fn epoch_summary_can_publish_with_explicit_gaps() {
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

    let summary = build_epoch_summary(
        &epoch_config(
            &["store-001", "store-002"],
            LakeStragglerPolicy::PublishWithGaps { grace_ms: 30_000 },
        ),
        &[envelope],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::CompleteWithGaps);
    assert_eq!(summary.complete_source_count, 1);
    assert_eq!(summary.missing_source_count, 1);
    let missing = summary
        .sources
        .iter()
        .find(|source| source.source_id == "store-002")
        .expect("missing source");
    assert_eq!(missing.state, LakeEpochSourceState::Missing);
    assert!(missing
        .gap_reason
        .as_deref()
        .is_some_and(|reason| reason.contains("published gap epoch")));
}

#[test]
fn wait_all_required_keeps_epoch_open_when_source_is_lagging() {
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

    let summary = build_epoch_summary(
        &epoch_config(
            &["store-001", "store-002"],
            LakeStragglerPolicy::WaitAllRequired,
        ),
        &[envelope],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::Open);
    assert_eq!(summary.missing_source_count, 1);
    assert_eq!(
        summary
            .sources
            .iter()
            .find(|source| source.source_id == "store-002")
            .expect("lagging source")
            .state,
        LakeEpochSourceState::Lagging
    );
    assert_eq!(
        summary.verification.status,
        LakeEpochVerificationStatus::Unknown
    );
}
