use super::*;
use trellara_protocol::Operation;

#[test]
fn parses_test_decoding_insert_change() {
    let event = parse_test_decoding_event(
            "0/197A0B8",
            "table public.sales: INSERT: id[text]:'sale-1' store_id[text]:'store-1' amount_cents[text]:'1299'",
        )
        .expect("parse insert");

    let LogicalEvent::Change {
        relation,
        operation,
        before,
        after,
        ..
    } = event
    else {
        panic!("expected change");
    };
    assert_eq!(relation.schema, "public");
    assert_eq!(relation.table, "sales");
    assert_eq!(operation, Operation::Insert);
    assert!(before.is_none());
    let after = after.expect("after image");
    assert_eq!(after.columns.len(), 3);
    assert_eq!(after.columns[0].name, "id");
    assert!(after.columns[0].is_key);
}

#[test]
fn parses_test_decoding_update_change() {
    let event = parse_test_decoding_event(
            "0/197A358",
            "table public.sales: UPDATE: old-key: id[text]:'sale-1' amount_cents[text]:'100' new-tuple: id[text]:'sale-1' amount_cents[text]:'101'",
        )
        .expect("parse update");

    let LogicalEvent::Change {
        operation,
        before,
        after,
        ..
    } = event
    else {
        panic!("expected change");
    };
    assert_eq!(operation, Operation::Update);
    assert_eq!(
        before.expect("before").columns[1].text_value,
        "100".to_string()
    );
    assert_eq!(
        after.expect("after").columns[1].text_value,
        "101".to_string()
    );
}

#[test]
fn parses_test_decoding_delete_change() {
    let event = parse_test_decoding_event(
        "0/197A450",
        "table public.sales: DELETE: id[text]:'sale-1' amount_cents[text]:'101'",
    )
    .expect("parse delete");

    let LogicalEvent::Change {
        operation,
        before,
        after,
        ..
    } = event
    else {
        panic!("expected change");
    };
    assert_eq!(operation, Operation::Delete);
    assert_eq!(before.expect("before").columns[0].text_value, "sale-1");
    assert!(after.is_none());
}

#[test]
fn parses_test_decoding_truncate_change() {
    let event = parse_test_decoding_event("0/197A500", "table public.sales: TRUNCATE: (no-flags)")
        .expect("parse truncate");

    let LogicalEvent::Truncate { relations, .. } = event else {
        panic!("expected truncate");
    };
    assert_eq!(relations, vec![RelationId::new(0, "public", "sales")]);
}

#[test]
fn parses_test_decoding_multi_relation_truncate_change() {
    let event = parse_test_decoding_event(
        "0/197A520",
        "table public.sales, public.sale_items: TRUNCATE: restart_seqs cascade",
    )
    .expect("parse truncate");

    let LogicalEvent::Truncate { relations, .. } = event else {
        panic!("expected truncate");
    };
    assert_eq!(
        relations,
        vec![
            RelationId::new(0, "public", "sales"),
            RelationId::new(0, "public", "sale_items"),
        ]
    );
}

#[test]
fn parses_test_decoding_transaction_boundaries() {
    assert!(matches!(
        parse_test_decoding_event("0/1", "BEGIN 755").expect("begin"),
        LogicalEvent::Begin {
            transaction_id,
            begin_lsn
        } if transaction_id == "755" && begin_lsn == "0/1"
    ));
    assert!(matches!(
        parse_test_decoding_event("0/2", "COMMIT 755").expect("commit"),
        LogicalEvent::Commit {
            commit_lsn,
            commit_timestamp_ms: 0,
        } if commit_lsn == "0/2"
    ));
}
