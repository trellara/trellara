use super::fixtures::{default_envelope, envelope_with};
use super::*;

#[test]
fn envelope_round_trips_with_checksum() {
    let mut change = sample_change(1);
    change.after = Some(RowImage::new(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::binary("receipt", 17, vec![1, 2, 3, 4], false),
    ]));
    let envelope = envelope_with("tx-1", "0/16B6B00", "0/16B6C50", vec![change]);

    let encoded = envelope.encode_checked().expect("encode envelope");
    let decoded = TransactionEnvelope::decode_checked(&encoded).expect("decode envelope");

    assert_eq!(decoded.source_id, "source-a");
    assert_eq!(decoded.changes.len(), 1);
    let columns = &decoded.changes[0]
        .after
        .as_ref()
        .expect("after image")
        .columns;
    assert_eq!(
        columns[1],
        ColumnValue::binary("receipt", 17, vec![1, 2, 3, 4], false)
    );
    assert_eq!(decoded.checksum, envelope.checksum);
}

#[test]
fn checksum_detects_mutation() {
    let mut envelope = default_envelope();

    envelope.commit_lsn = "0/DEADBEEF".to_string();

    assert!(matches!(
        envelope.verify_checksum(),
        Err(ProtocolError::ChecksumMismatch { .. })
    ));
}
