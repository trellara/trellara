use super::*;

#[test]
fn postgres_snapshot_query_quotes_identifiers() {
    assert_eq!(
        quote_table(&RelationId::new(42, "public", "sales")),
        "\"public\".\"sales\""
    );
    assert_eq!(quote_ident("odd\"name"), "\"odd\"\"name\"");
}

#[test]
fn snapshot_query_includes_configured_row_filter() {
    assert_eq!(
        snapshot_select_sql(
            &RelationId::new(42, "public", "sales"),
            "id",
            Some("region = 'west'")
        ),
        "select \"id\"::text as trellara_primary_key, (to_jsonb(t) - $1::text[])::text as trellara_row from \"public\".\"sales\" as t where (region = 'west') order by \"id\"::text"
    );
}

#[test]
fn reseed_clear_sql_deletes_only_filtered_rows() {
    assert_eq!(
        reseed_clear_sql(
            &RelationId::new(42, "public", "sales"),
            Some("region = 'west'")
        ),
        "delete from \"public\".\"sales\" where (region = 'west')"
    );
    assert_eq!(
        reseed_clear_sql(&RelationId::new(42, "public", "sales"), None),
        "truncate table \"public\".\"sales\""
    );
}

#[test]
fn reseed_insert_sql_quotes_columns() {
    assert_eq!(
        reseed_insert_sql(
            &RelationId::new(42, "public", "sales"),
            &[
                CopyableColumn {
                    name: "id".to_string(),
                    type_name: "text".to_string(),
                },
                CopyableColumn {
                    name: "odd\"name".to_string(),
                    type_name: "text".to_string(),
                },
            ]
        ),
        "insert into \"public\".\"sales\" (\"id\", \"odd\"\"name\") values ($1, $2)"
    );
}

#[test]
fn reseed_insert_sql_decodes_bytea_columns() {
    assert_eq!(
        reseed_insert_sql(
            &RelationId::new(42, "public", "sales"),
            &[
                CopyableColumn {
                    name: "id".to_string(),
                    type_name: "text".to_string(),
                },
                CopyableColumn {
                    name: "receipt_bytes".to_string(),
                    type_name: "bytea".to_string(),
                },
            ]
        ),
        "insert into \"public\".\"sales\" (\"id\", \"receipt_bytes\") values ($1, decode($2, 'hex'))"
    );
}

#[test]
fn source_reseed_select_sql_encodes_bytea_columns() {
    assert_eq!(
        source_reseed_select_sql(
            &RelationId::new(42, "public", "sales"),
            "id",
            &[
                CopyableColumn {
                    name: "id".to_string(),
                    type_name: "text".to_string(),
                },
                CopyableColumn {
                    name: "receipt_bytes".to_string(),
                    type_name: "bytea".to_string(),
                },
            ],
            None,
        ),
        "select \"id\"::text, encode(\"receipt_bytes\", 'hex') from \"public\".\"sales\" order by \"id\"::text"
    );
}

#[test]
fn source_reseed_select_sql_includes_row_filter() {
    assert_eq!(
        source_reseed_select_sql(
            &RelationId::new(42, "public", "sales"),
            "id",
            &[CopyableColumn {
                name: "id".to_string(),
                type_name: "text".to_string(),
            }],
            Some("region = 'west'"),
        ),
        "select \"id\"::text from \"public\".\"sales\" where (region = 'west') order by \"id\"::text"
    );
}
