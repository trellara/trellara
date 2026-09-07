use super::*;

#[test]
fn relation_metadata_records_stable_identity_contract() {
    let metadata =
        native_relation_metadata(16_384, "public", "orders", 42).expect("relation metadata");

    assert_eq!(metadata.relation_oid, 16_384);
    assert_eq!(metadata.namespace, "public");
    assert_eq!(metadata.relation_name, "orders");
    assert_eq!(metadata.schema_version_fingerprint, 42);
    assert_eq!(
        metadata.required_replica_identity,
        REQUIRED_REPLICA_IDENTITY
    );
}

#[test]
fn relation_metadata_rejects_ambiguous_identity() {
    assert_eq!(
        native_relation_metadata(0, "public", "orders", 42),
        Err(NativeRelationMetadataError::InvalidRelationOid)
    );
    assert_eq!(
        native_relation_metadata(16_384, " ", "orders", 42),
        Err(NativeRelationMetadataError::BlankNamespace)
    );
    assert_eq!(
        native_relation_metadata(16_384, "public", "", 42),
        Err(NativeRelationMetadataError::BlankRelationName)
    );
    assert_eq!(
        native_relation_metadata(16_384, " public ", "orders", 42),
        Err(NativeRelationMetadataError::PaddedNamespace)
    );
    assert_eq!(
        native_relation_metadata(16_384, "public", " orders ", 42),
        Err(NativeRelationMetadataError::PaddedRelationName)
    );
    assert_eq!(
        native_relation_metadata(16_384, "public", "orders", 0),
        Err(NativeRelationMetadataError::MissingSchemaFingerprint)
    );
}
