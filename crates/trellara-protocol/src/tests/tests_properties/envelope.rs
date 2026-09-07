use super::*;

proptest! {
    #[test]
    fn envelope_roundtrip_property_preserves_checksum(change_count in 1_u32..64) {
        let envelope = generated_envelope(change_count);

        let encoded = envelope.encode_checked().expect("encode envelope");
        let decoded = TransactionEnvelope::decode_checked(&encoded).expect("decode envelope");

        prop_assert_eq!(decoded.transaction_id.as_str(), "tx-generated");
        prop_assert_eq!(decoded.changes.len(), change_count as usize);
        prop_assert_eq!(decoded.checksum, envelope.checksum);
        prop_assert!(decoded.verify_checksum().is_ok());
    }

    #[test]
    fn envelope_roundtrip_property_preserves_binary_column(
        payload in prop::collection::vec(any::<u8>(), 0..512),
    ) {
        let mut change = sample_change(1);
        change.transaction_id = "tx-binary".to_string();
        change.idempotency_key = idempotency_key("source-a", "0/16B6C50", "tx-binary", 1);
        change.after = Some(RowImage::new(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue::binary("receipt", 17, payload.clone(), false),
        ]));
        let envelope = TransactionEnvelope::strict(StrictEnvelope {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            transaction_id: "tx-binary".to_string(),
            begin_lsn: "0/16B6B00".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 1_786_420_000_000,
            changes: vec![change],
        });

        let encoded = envelope.encode_checked().expect("encode envelope");
        let decoded = TransactionEnvelope::decode_checked(&encoded).expect("decode envelope");
        let columns = &decoded.changes[0]
            .after
            .as_ref()
            .expect("after image")
            .columns;

        prop_assert_eq!(columns[1].value_kind, ValueKind::Binary as i32);
        prop_assert_eq!(columns[1].binary_value.as_ref(), payload.as_slice());
        prop_assert_eq!(columns[1].text_value.as_str(), "");
        prop_assert!(decoded.verify_checksum().is_ok());
    }

    #[test]
    fn checksum_property_rejects_binary_column_mutation(
        payload in prop::collection::vec(any::<u8>(), 1..512),
    ) {
        let mut change = sample_change(1);
        change.transaction_id = "tx-binary".to_string();
        change.idempotency_key = idempotency_key("source-a", "0/16B6C50", "tx-binary", 1);
        change.after = Some(RowImage::new(vec![
            ColumnValue::text("id", 23, "sale-1", true),
            ColumnValue::binary("receipt", 17, payload, false),
        ]));
        let envelope = TransactionEnvelope::strict(StrictEnvelope {
            source_id: "source-a".to_string(),
            database_id: "retail".to_string(),
            dataset_id: "sales".to_string(),
            transaction_id: "tx-binary".to_string(),
            begin_lsn: "0/16B6B00".to_string(),
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 1_786_420_000_000,
            changes: vec![change],
        });

        let encoded = envelope.encode_checked().expect("encode envelope");
        let mut decoded = TransactionEnvelope::decode_checked(&encoded).expect("decode envelope");
        let binary_column = &mut decoded.changes[0]
            .after
            .as_mut()
            .expect("after image")
            .columns[1];
        let mut mutated = binary_column.binary_value.to_vec();
        mutated[0] ^= 0xFF;
        binary_column.binary_value = mutated.into();

        match decoded.verify_checksum() {
            Err(ProtocolError::ChecksumMismatch { .. }) => {}
            other => prop_assert!(false, "expected checksum mismatch, got {other:?}"),
        }
    }
}
