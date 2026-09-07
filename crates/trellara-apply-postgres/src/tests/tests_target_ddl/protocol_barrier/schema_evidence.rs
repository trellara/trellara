use super::*;

#[test]
fn target_ddl_barrier_accepts_matching_schema_version_evidence() {
    let mut envelope = ddl_envelope();
    envelope.schema_versions = vec![RelationSchemaVersion {
        relation: Some(relation()),
        version: 67_890,
    }];
    envelope.finalize_checksum();

    let barrier = target_ddl_barrier_from_envelope(&envelope)
        .expect("DDL barrier")
        .expect("DDL events");

    assert_eq!(barrier.schema_version, "schema-fingerprint:67890");
}

#[test]
fn target_ddl_barrier_rejects_schema_version_evidence_mismatch() {
    let mut envelope = ddl_envelope();
    envelope.schema_versions = vec![RelationSchemaVersion {
        relation: Some(relation()),
        version: 12_345,
    }];
    envelope.finalize_checksum();

    let error = target_ddl_barrier_from_envelope(&envelope).expect_err("schema mismatch");

    assert!(matches!(
        error,
        ApplyError::DdlSchemaVersionMismatch {
            relation,
            expected: 67_890,
            actual: 12_345,
        } if relation == "public.sales"
    ));
}

#[test]
fn target_ddl_barrier_rejects_mismatch_for_final_ddl_relation_only() {
    let mut envelope = ddl_envelope();
    envelope.ddl_events = vec![
        DdlEvent::additive_column(
            "tx-ddl",
            1,
            relation(),
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
            10,
            90_000,
        ),
        DdlEvent::additive_column(
            "tx-ddl",
            2,
            RelationId::new(43, "public", "orders"),
            "ALTER TABLE \"public\".\"orders\" ADD COLUMN \"audit_note\" text;",
            20,
            12_000,
        ),
    ];
    envelope.schema_versions = vec![
        RelationSchemaVersion {
            relation: Some(relation()),
            version: 90_000,
        },
        RelationSchemaVersion {
            relation: Some(RelationId::new(43, "public", "orders")),
            version: 90_000,
        },
    ];
    envelope.finalize_checksum();

    let error = target_ddl_barrier_from_envelope(&envelope).expect_err("schema mismatch");

    assert!(matches!(
        error,
        ApplyError::DdlSchemaVersionMismatch {
            relation,
            expected: 12_000,
            actual: 90_000,
        } if relation == "public.orders"
    ));
}
