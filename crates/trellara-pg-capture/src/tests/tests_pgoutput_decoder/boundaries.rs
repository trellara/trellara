use super::super::*;
use trellara_protocol::Operation;

#[test]
fn pgoutput_decoder_parses_delete_truncate_and_commit_boundaries() {
    let mut decoder = PgOutputDecoder::default();
    decoder
        .decode(&pgoutput_relation_message(
            16_384,
            "public",
            "sales",
            b'f',
            &[("id", 25, true), ("amount_cents", 25, false)],
        ))
        .expect("relation");

    assert_eq!(
        decoder
            .decode(&pgoutput_begin_message(0x16B6B00, 42, 123))
            .expect("begin"),
        Some(LogicalEvent::Begin {
            transaction_id: "42".to_string(),
            begin_lsn: "0/16B6B00".to_string(),
        })
    );

    let delete = decoder
        .decode(&pgoutput_delete_message(
            16_384,
            b'O',
            &[TupleValue::text("sale-1"), TupleValue::text("1299")],
        ))
        .expect("delete")
        .expect("event");
    assert!(matches!(
        delete,
        LogicalEvent::Change {
            operation: Operation::Delete,
            ..
        }
    ));

    assert_eq!(
        decoder
            .decode(&pgoutput_truncate_message(&[16_384]))
            .expect("truncate"),
        Some(LogicalEvent::Truncate {
            transaction_id: None,
            relations: vec![RelationId::new(16_384, "public", "sales")]
        })
    );

    assert_eq!(
        decoder
            .decode(&pgoutput_commit_message(0x16B6C00, 0x16B6C50, 123_000))
            .expect("commit"),
        Some(LogicalEvent::Commit {
            commit_lsn: "0/16B6C50".to_string(),
            commit_timestamp_ms: 946_684_800_123,
        })
    );
}
