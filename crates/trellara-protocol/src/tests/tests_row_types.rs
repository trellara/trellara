use prost::Message;

use crate::{ColumnValue, RelationId, RelationSchemaVersion, RowImage, ValueKind};

#[test]
fn relation_display_name_uses_schema_and_table() {
    assert_eq!(
        RelationId::new(42, "tenant_a", "orders").display_name(),
        "tenant_a.orders"
    );
}

#[test]
fn row_images_keep_public_constructor_and_value_kinds() {
    let row = RowImage::new(vec![
        ColumnValue::text("id", 23, "sale-1", true),
        ColumnValue::binary("receipt", 17, vec![1, 2, 3], false),
        ColumnValue::null("note", 25, false),
        ColumnValue::unchanged_toast("large_blob", 17, false),
    ]);

    assert_eq!(row.columns[0].value_kind, ValueKind::Text as i32);
    assert_eq!(row.columns[1].binary_value.as_ref(), &[1, 2, 3]);
    assert_eq!(row.columns[2].value_kind, ValueKind::Null as i32);
    assert_eq!(row.columns[3].value_kind, ValueKind::UnchangedToast as i32);
}

#[test]
fn row_types_round_trip_through_prost_wire_format() {
    let version = RelationSchemaVersion {
        relation: Some(RelationId::new(42, "public", "sales")),
        version: 7,
    };

    let mut bytes = Vec::new();
    version.encode(&mut bytes).expect("encode schema version");

    let decoded = RelationSchemaVersion::decode(bytes.as_slice()).expect("decode schema version");
    assert_eq!(decoded, version);
}
