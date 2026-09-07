use super::LOAD_RELATIONS_SQL;

#[test]
fn load_relations_query_preserves_native_postgres_oid_type() {
    assert!(LOAD_RELATIONS_SQL.contains("select c.oid as oid"));
    assert!(!LOAD_RELATIONS_SQL.contains("c.oid::int4"));
}

#[test]
fn load_relations_query_casts_internal_char_replica_identity_to_text() {
    assert!(LOAD_RELATIONS_SQL.contains("c.relreplident::text as replica_identity"));
}
