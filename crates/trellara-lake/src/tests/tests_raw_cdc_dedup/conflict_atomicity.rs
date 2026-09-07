use super::super::*;
use super::*;
use trellara_protocol::idempotency_key;

#[test]
fn conflicting_transaction_identity_does_not_poison_new_idempotency_key() {
    let first = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    let mut conflicting = envelope("store-001", "tx-1", "0/16B6C50", "sale-2", "99");
    conflicting.changes[0].idempotency_key = idempotency_key("store-001", "0/16B6C70", "tx-2", 1);
    conflicting.finalize_checksum();
    let mut valid_after_conflict = envelope("store-001", "tx-2", "0/16B6C70", "sale-2", "99");
    valid_after_conflict.changes[0].idempotency_key =
        conflicting.changes[0].idempotency_key.clone();
    valid_after_conflict.finalize_checksum();
    let mut ledger = RawCdcDedupLedger::default();

    ledger.record(&first).expect("first record");
    assert!(matches!(
        ledger.record(&conflicting),
        Err(LakeError::ConflictingDuplicate { .. })
    ));

    assert_eq!(
        ledger
            .record(&valid_after_conflict)
            .expect("valid independent transaction"),
        RawCdcDedupOutcome::New {
            transaction_id: "store-001:postgres:retail:tx-2:0/16B6C70".to_string(),
        }
    );
}

#[test]
fn conflicting_idempotency_key_does_not_partially_record_clean_changes() {
    let first = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    let mut conflicting = two_change_envelope("store-001", "tx-2", "0/16B6C70");
    let clean_key = conflicting.changes[0].idempotency_key.clone();
    conflicting.changes[1].idempotency_key = first.changes[0].idempotency_key.clone();
    conflicting.finalize_checksum();
    let mut valid_after_conflict = envelope("store-001", "tx-3", "0/16B6C90", "sale-2", "99");
    valid_after_conflict.changes[0].idempotency_key = clean_key;
    valid_after_conflict.finalize_checksum();
    let mut ledger = RawCdcDedupLedger::default();

    ledger.record(&first).expect("first record");
    assert!(matches!(
        ledger.record(&conflicting),
        Err(LakeError::ConflictingDuplicate { .. })
    ));

    assert_eq!(
        ledger
            .record(&valid_after_conflict)
            .expect("clean key remains available"),
        RawCdcDedupOutcome::New {
            transaction_id: "store-001:postgres:retail:tx-3:0/16B6C90".to_string(),
        }
    );
}
