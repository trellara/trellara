use super::*;
use trellara_protocol::{ColumnValue, Operation, ValueKind};

#[test]
fn pgoutput_decoder_parses_update_key_and_omits_unchanged_toast() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("description", 25, false)],
        ))
        .expect("relation");

    let event = decoder
        .decode(&pgoutput_update_message(
            16_384,
            Some((
                b'K',
                vec![TupleValue::text("sale-1"), TupleValue::unchanged()],
            )),
            &[TupleValue::text("sale-1"), TupleValue::text("fresh")],
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
            let before = before.expect("before").columns;
            assert_eq!(before.len(), 2);
            assert_eq!(before[0], ColumnValue::text("id", 25, "sale-1", true));
            assert_eq!(before[1].name, "description");
            assert_eq!(before[1].value_kind, ValueKind::UnchangedToast as i32);
            let after = after.expect("after").columns;
            assert_eq!(after.len(), 2);
            assert_eq!(
                after[1],
                ColumnValue::text("description", 25, "fresh", false)
            );
        }
        other => panic!("unexpected event {other:?}"),
    }
}

#[test]
fn pgoutput_decoder_rejects_omitted_unchanged_key_column() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("description", 25, false)],
        ))
        .expect("relation");

    let error = decoder
        .decode(&pgoutput_update_message(
            16_384,
            Some((b'K', vec![TupleValue::unchanged(), TupleValue::unchanged()])),
            &[TupleValue::text("sale-1"), TupleValue::text("fresh")],
        ))
        .expect_err("omitted key fails closed");

    assert!(matches!(error, CaptureError::PgOutputParse(message) if
        message.contains("key column id")
            && message.contains("omitted as unchanged TOAST")
            && message.contains("public.sales")
    ));
}

#[test]
fn pgoutput_decoder_rejects_delete_with_omitted_unchanged_key_column() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'd',
            &[("id", 25, true), ("description", 25, false)],
        ))
        .expect("relation");

    let error = decoder
        .decode(&pgoutput_delete_message(
            16_384,
            b'K',
            &[TupleValue::unchanged(), TupleValue::unchanged()],
        ))
        .expect_err("omitted delete key fails closed");

    assert!(matches!(error, CaptureError::PgOutputParse(message) if
        message.contains("key column id")
            && message.contains("cannot safely identify UPDATE/DELETE rows")
    ));
}
