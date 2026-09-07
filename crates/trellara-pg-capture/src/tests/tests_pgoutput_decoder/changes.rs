use super::super::*;
use trellara_protocol::{ColumnValue, Operation, ReplicaIdentity, ValueKind};

#[test]
fn pgoutput_decoder_tracks_relation_and_insert_tuple() {
    let mut decoder = PgOutputDecoder::default();
    assert!(matches!(
        decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[
                ("id", 25, true),
                ("amount_cents", 25, false),
                ("receipt", 17, false),
                ("note", 25, false),
            ],
        ))
        .expect("relation"),
        Some(LogicalEvent::RelationMetadata {
            relation,
            schema_fingerprint,
        }) if relation == RelationId::new(16_384, "public", "sales") && schema_fingerprint > 0
    ));

    let event = decoder
        .decode(&pgoutput_insert_message(
            16_384,
            &[
                TupleValue::text("sale-1"),
                TupleValue::text("1299"),
                TupleValue::Binary(b"\x01\x02"),
                TupleValue::Null,
            ],
        ))
        .expect("insert")
        .expect("event");

    match event {
        LogicalEvent::Change {
            transaction_id,
            relation,
            operation,
            replica_identity,
            before,
            after,
        } => {
            assert_eq!(transaction_id, None);
            assert_eq!(relation, RelationId::new(16_384, "public", "sales"));
            assert_eq!(operation, Operation::Insert);
            assert_eq!(replica_identity, ReplicaIdentity::Default);
            assert_eq!(before, None);
            let columns = after.expect("after").columns;
            assert_eq!(columns[0], ColumnValue::text("id", 25, "sale-1", true));
            assert_eq!(
                columns[1],
                ColumnValue::text("amount_cents", 25, "1299", false)
            );
            assert_eq!(columns[2].name, "receipt");
            assert_eq!(columns[2].value_kind, ValueKind::Binary as i32);
            assert_eq!(columns[2].binary_value.as_ref(), b"\x01\x02");
            assert_eq!(columns[3], ColumnValue::null("note", 25, false));
        }
        other => panic!("unexpected event {other:?}"),
    }
}

#[test]
fn pgoutput_decoder_preserves_non_key_unchanged_toast_sentinel() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("receipt", 17, false)],
        ))
        .expect("relation");

    let event = decoder
        .decode(&pgoutput_update_message(
            16_384,
            Some((
                b'K',
                vec![TupleValue::text("sale-1"), TupleValue::unchanged()],
            )),
            &[TupleValue::text("sale-1"), TupleValue::unchanged()],
        ))
        .expect("update")
        .expect("event");

    match event {
        LogicalEvent::Change {
            operation,
            before,
            after,
            ..
        } => {
            assert_eq!(operation, Operation::Update);
            let before_columns = before.expect("before").columns;
            let after_columns = after.expect("after").columns;
            assert_eq!(
                before_columns[1],
                ColumnValue::unchanged_toast("receipt", 17, false)
            );
            assert_eq!(
                after_columns[1],
                ColumnValue::unchanged_toast("receipt", 17, false)
            );
        }
        other => panic!("unexpected event {other:?}"),
    }
}

#[test]
fn pgoutput_decoder_rejects_key_column_unchanged_toast() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("receipt", 17, false)],
        ))
        .expect("relation");

    let error = decoder
        .decode(&pgoutput_update_message(
            16_384,
            Some((b'K', vec![TupleValue::unchanged(), TupleValue::unchanged()])),
            &[TupleValue::text("sale-1"), TupleValue::unchanged()],
        ))
        .expect_err("key column unchanged TOAST should fail closed");

    assert!(matches!(error, CaptureError::PgOutputParse(message)
            if message.contains("key column id")
                && message.contains("omitted as unchanged TOAST")));
}
