use super::super::*;
use super::*;
use trellara_protocol::{idempotency_key, ProtocolError};

#[test]
fn raw_cdc_dedup_ledger_marks_replayed_transaction_duplicate() {
    let envelope = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    let mut ledger = RawCdcDedupLedger::default();

    assert_eq!(
        ledger.record(&envelope).expect("first record"),
        RawCdcDedupOutcome::New {
            transaction_id: "store-001:postgres:retail:tx-1:0/16B6C50".to_string(),
        }
    );
    assert_eq!(
        ledger.record(&envelope).expect("duplicate record"),
        RawCdcDedupOutcome::Duplicate
    );
}

#[test]
fn raw_cdc_dedup_ledger_rejects_conflicting_idempotency_key() {
    let first = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    let mut conflicting = envelope("store-001", "tx-2", "0/16B6C70", "sale-1", "99");
    conflicting.changes[0].idempotency_key = first.changes[0].idempotency_key.clone();
    conflicting.finalize_checksum();
    let mut ledger = RawCdcDedupLedger::default();

    ledger.record(&first).expect("first record");

    assert!(matches!(
        ledger.record(&conflicting),
        Err(LakeError::ConflictingDuplicate { idempotency_key })
            if idempotency_key == first.changes[0].idempotency_key
    ));
}

#[test]
fn raw_cdc_dedup_ledger_rejects_conflicting_transaction_identity() {
    let first = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    let conflicting = envelope("store-001", "tx-1", "0/16B6C50", "sale-2", "99");
    let mut ledger = RawCdcDedupLedger::default();

    ledger.record(&first).expect("first record");

    assert!(matches!(
        ledger.record(&conflicting),
        Err(LakeError::ConflictingDuplicate { idempotency_key })
            if idempotency_key == conflicting.changes[0].idempotency_key
    ));
}

#[test]
fn raw_cdc_transaction_identity_includes_database_and_dataset() {
    let retail = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    let mut wholesale = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    wholesale.dataset_id = "wholesale".to_string();
    wholesale.changes[0].idempotency_key = idempotency_key("store-001", "0/16B6C50", "tx-1", 1);
    wholesale.finalize_checksum();
    let mut ledger = RawCdcDedupLedger::default();

    assert_eq!(
        ledger.record(&retail).expect("retail record"),
        RawCdcDedupOutcome::New {
            transaction_id: "store-001:postgres:retail:tx-1:0/16B6C50".to_string(),
        }
    );
    assert_eq!(
        ledger.record(&wholesale).expect("wholesale record"),
        RawCdcDedupOutcome::New {
            transaction_id: "store-001:postgres:wholesale:tx-1:0/16B6C50".to_string(),
        }
    );
}

#[test]
fn raw_cdc_dedup_rejects_missing_transaction_identity() {
    let mut envelope = envelope("store-001", "tx-1", "0/16B6C50", "sale-1", "10");
    envelope.transaction_id.clear();
    envelope.finalize_checksum();
    let mut ledger = RawCdcDedupLedger::default();

    assert!(matches!(
        ledger.record(&envelope),
        Err(LakeError::InvalidEnvelope(
            ProtocolError::MissingEnvelopeField {
                field: "transaction_id"
            }
        ))
    ));
}
