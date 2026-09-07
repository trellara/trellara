use super::*;

#[test]
fn epoch_summary_ignores_other_datasets() {
    let summary = build_epoch_summary(
        &epoch_config(
            &["source-a"],
            LakeStragglerPolicy::PublishWithGaps { grace_ms: 0 },
        ),
        &[envelope_for_other_dataset(vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )])],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::CompleteWithGaps);
    assert_eq!(summary.transaction_count, 0);
    assert_eq!(summary.change_count, 0);
}

#[test]
fn epoch_summary_ignores_sources_outside_required_fleet_scope() {
    let required = envelope_for(
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
    let unexpected = envelope_for(
        "store-999",
        "tx-2",
        "0/16B6D00",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-999", "999", "2026-01-01")),
        )],
    );

    let summary = build_epoch_summary(
        &epoch_config(&["store-001"], LakeStragglerPolicy::WaitAllRequired),
        &[required.clone(), unexpected],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::Complete);
    assert_eq!(summary.required_source_count, 1);
    assert_eq!(summary.complete_source_count, 1);
    assert_eq!(summary.transaction_count, 1);
    assert_eq!(summary.change_count, 1);
    assert_eq!(summary.checksum_rollup, required.checksum);
    assert_eq!(summary.sources.len(), 1);
    assert_eq!(summary.sources[0].source_id, "store-001");
    assert_eq!(summary.tables[0].transaction_count, 1);
}

#[test]
fn epoch_summary_includes_all_dataset_sources_when_unscoped() {
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
        "store-999",
        "tx-2",
        "0/16B6D00",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-999", "999", "2026-01-01")),
        )],
    );

    let summary = build_epoch_summary(
        &epoch_config(&[], LakeStragglerPolicy::WaitAllRequired),
        &[first.clone(), second.clone()],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::Complete);
    assert_eq!(summary.required_source_count, 0);
    assert_eq!(summary.complete_source_count, 2);
    assert_eq!(summary.transaction_count, 2);
    assert_eq!(summary.change_count, 2);
    assert_eq!(summary.checksum_rollup, first.checksum ^ second.checksum);
    assert_eq!(
        summary
            .sources
            .iter()
            .map(|source| source.source_id.as_str())
            .collect::<Vec<_>>(),
        vec!["store-001", "store-999"]
    );
}
