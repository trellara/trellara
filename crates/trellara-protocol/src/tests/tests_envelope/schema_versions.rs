use super::fixtures::{default_envelope, schema_version};
use super::*;

#[test]
fn checked_encoding_round_trips_schema_version_evidence() {
    let mut envelope = default_envelope();
    envelope.schema_versions = vec![schema_version(67_890)];
    envelope.finalize_checksum();

    let decoded = TransactionEnvelope::decode_checked(
        &envelope.encode_checked().expect("encode schema evidence"),
    )
    .expect("decode schema evidence");

    assert_eq!(decoded.schema_versions.len(), 1);
    assert_eq!(decoded.schema_versions[0].version, 67_890);
    assert_eq!(
        decoded.schema_versions[0]
            .relation
            .as_ref()
            .expect("relation")
            .display_name(),
        "public.sales"
    );
}

#[test]
fn checked_encoding_rejects_invalid_schema_version_evidence() {
    for (schema_version, expected_reason) in [
        (
            RelationSchemaVersion {
                relation: None,
                version: 67_890,
            },
            "relation metadata is required",
        ),
        (
            RelationSchemaVersion {
                relation: Some(RelationId::new(16_384, "public", "sales")),
                version: 0,
            },
            "version must be greater than zero",
        ),
    ] {
        let mut envelope = default_envelope();
        envelope.schema_versions = vec![schema_version];
        envelope.finalize_checksum();

        assert!(matches!(
            envelope.encode_checked(),
            Err(ProtocolError::InvalidSchemaVersionEvidence { reason, .. })
                if reason == expected_reason
        ));
    }
}

#[test]
fn checked_encoding_rejects_duplicate_schema_version_evidence() {
    let mut envelope = default_envelope();
    envelope.schema_versions = vec![schema_version(67_890), schema_version(98_765)];
    envelope.finalize_checksum();

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidSchemaVersionEvidence { relation, reason })
            if relation == "public.sales" && reason == "duplicate relation schema version"
    ));
}

#[test]
fn checked_encoding_rejects_ambiguous_schema_version_relation_names() {
    let mut envelope = default_envelope();
    envelope.schema_versions = vec![
        schema_version(67_890),
        RelationSchemaVersion {
            relation: Some(RelationId::new(98_765, "public", "sales")),
            version: 98_765,
        },
    ];
    envelope.finalize_checksum();

    assert!(matches!(
        envelope.encode_checked(),
        Err(ProtocolError::InvalidSchemaVersionEvidence { relation, reason })
            if relation == "public.sales" && reason == "duplicate relation schema version"
    ));
}
