use super::*;

#[test]
fn target_ddl_barrier_from_envelope_uses_commit_boundary() {
    let envelope = ddl_envelope();
    let barrier = target_ddl_barrier_from_envelope(&envelope)
        .expect("DDL barrier")
        .expect("DDL events");

    assert_eq!(barrier.source_id, "source");
    assert_eq!(barrier.dataset_id, "sales");
    assert_eq!(
        barrier.barrier_id,
        "source:retail:sales:tx-ddl:0/16B6C50:ddl"
    );
    assert_eq!(barrier.barrier_lsn, "0/16B6C50");
    assert_eq!(barrier.schema_version, "schema-fingerprint:67890");
    assert!(barrier
        .cdc_transaction_boundary
        .contains(DDL_PROPAGATION_CDC_BOUNDARY));
    assert!(barrier.cdc_transaction_boundary.contains(
        "propagation_decisions=auto_apply:1,manual_review:0,unsupported:0,target_ack_required:1"
    ));
    assert!(barrier
        .cdc_transaction_boundary
        .contains("propagation_policy_sha256="));
    assert_eq!(barrier.required_sinks, vec!["target_postgres"]);
    assert!(!barrier.requires_global_partition_pause);
}

#[test]
fn target_ddl_barrier_boundary_counts_only_releasable_target_ack_requirements() {
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
        DdlEvent::manual_review(
            "tx-ddl",
            2,
            DdlOperation::RenameColumn,
            relation(),
            "ALTER TABLE \"public\".\"sales\" RENAME COLUMN \"discount_code\" TO \"coupon_code\";",
            90_000,
            91_000,
        ),
        DdlEvent::classified(DdlEventInput {
            transaction_id: "tx-ddl".to_string(),
            total_order: 3,
            operation: DdlOperation::Other,
            relation: relation(),
            statement: "ALTER TABLE \"public\".\"sales\" ALTER COLUMN \"coupon_code\" SET STORAGE EXTERNAL;"
                .to_string(),
            schema_fingerprint_before: 91_000,
            schema_fingerprint_after: 92_000,
            target_auto_apply: false,
            release_gate: String::new(),
        }),
    ];
    envelope.schema_versions = vec![RelationSchemaVersion {
        relation: Some(relation()),
        version: 92_000,
    }];
    envelope.finalize_checksum();

    let barrier = target_ddl_barrier_from_envelope(&envelope)
        .expect("DDL barrier")
        .expect("DDL events");

    assert!(barrier.cdc_transaction_boundary.contains(
        "propagation_decisions=auto_apply:1,manual_review:1,unsupported:1,target_ack_required:2"
    ));
}

#[test]
fn target_ddl_barrier_canonicalizes_commit_lsn_boundary() {
    let mut envelope = ddl_envelope();
    envelope.commit_lsn = "00000000/016B6C50".to_string();
    envelope.finalize_checksum();

    let barrier = target_ddl_barrier_from_envelope(&envelope)
        .expect("DDL barrier")
        .expect("DDL events");

    assert_eq!(
        barrier.barrier_id,
        "source:retail:sales:tx-ddl:0/16B6C50:ddl"
    );
    assert_eq!(barrier.barrier_lsn, "0/16B6C50");
}

#[test]
fn target_ddl_barrier_schema_version_uses_final_ddl_event_order() {
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
            relation(),
            "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"audit_note\" text;",
            90_000,
            12_000,
        ),
    ];
    envelope.finalize_checksum();

    let barrier = target_ddl_barrier_from_envelope(&envelope)
        .expect("DDL barrier")
        .expect("DDL events");

    assert_eq!(barrier.schema_version, "schema-fingerprint:12000");
}
