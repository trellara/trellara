use super::*;

#[test]
fn duplicate_envelope_replay_is_idempotent_for_epoch_counts() {
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
        &epoch_config(&["store-001"], LakeStragglerPolicy::WaitAllRequired),
        &[envelope.clone(), envelope.clone()],
    )
    .expect("epoch summary");

    assert_eq!(summary.state, LakeCompletenessState::Complete);
    assert_eq!(summary.transaction_count, 1);
    assert_eq!(summary.change_count, 1);
    assert_eq!(summary.checksum_rollup, envelope.checksum);
}

#[test]
fn conflicting_duplicate_idempotency_key_fails_closed() {
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
    let mut conflicting = envelope_for(
        "store-001",
        "tx-2",
        "0/16B6C70",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "99", "2026-01-01")),
        )],
    );
    conflicting.changes[0].idempotency_key = first.changes[0].idempotency_key.clone();
    conflicting.finalize_checksum();

    let error = build_epoch_summary(
        &epoch_config(&["store-001"], LakeStragglerPolicy::WaitAllRequired),
        &[first.clone(), conflicting],
    )
    .expect_err("conflicting duplicate");

    assert!(matches!(
        error,
        LakeError::ConflictingDuplicate { idempotency_key }
            if idempotency_key == first.changes[0].idempotency_key
    ));
}
