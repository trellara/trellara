use super::*;

#[test]
fn plan_envelope_preserves_change_order() {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![
            insert_change(),
            update_change(),
            delete_change(),
            truncate_change(),
        ],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.finalize_checksum();

    let statements = plan_envelope(&envelope).expect("envelope plan");

    assert_eq!(statements.len(), 4);
    assert!(statements[0].sql.starts_with("insert"));
    assert!(statements[1].sql.starts_with("update"));
    assert!(statements[2].sql.starts_with("delete"));
    assert!(statements[3].sql.starts_with("truncate"));
}

#[test]
fn plan_envelope_accepts_empty_transaction_as_noop() {
    let envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-empty".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });

    let statements = plan_envelope(&envelope).expect("empty transaction plan");

    assert!(statements.is_empty());
    assert_eq!(
        envelope.partitioned_scale_decision(),
        PartitionedScaleDecision::EmptyTransaction
    );
}

#[test]
fn plan_envelope_rejects_ddl_events_without_barrier_release() {
    let mut update = update_change();
    update.total_order = 3;
    update.table_order = 3;
    update.partition_order = 3;
    update.idempotency_key = idempotency_key("source", "0/16B6C50", "tx-1", 3);

    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![insert_change(), update],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-1",
        2,
        relation(),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let error = plan_envelope(&envelope).expect_err("DDL barrier required");

    assert!(matches!(
        error,
        ApplyError::DdlBarrierRequired {
            transaction_id,
            boundary_kind: TransactionBoundaryKind::MixedDdlAndDml,
            partitioned_scale_decision: PartitionedScaleDecision::DdlBarrierRequired,
            reason,
        } if transaction_id == "tx-1"
            && reason == "mixed DDL and DML transaction must apply DDL barrier before partitioned DML replay"
    ));
}

#[test]
fn plan_envelope_reports_ddl_only_boundary_without_barrier_release() {
    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-ddl-only".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: Vec::new(),
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-ddl-only",
        1,
        relation(),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let error = plan_envelope(&envelope).expect_err("DDL barrier required");

    assert!(matches!(
        error,
        ApplyError::DdlBarrierRequired {
            transaction_id,
            boundary_kind: TransactionBoundaryKind::DdlOnly,
            partitioned_scale_decision: PartitionedScaleDecision::DdlBarrierRequired,
            reason,
        } if transaction_id == "tx-ddl-only"
            && reason == "DDL-only transaction must use the DDL barrier path"
    ));
}

#[test]
fn plan_envelope_accepts_dml_replay_after_ddl_barrier() {
    let mut update = update_change();
    update.total_order = 3;
    update.table_order = 3;
    update.partition_order = 3;
    update.idempotency_key = idempotency_key("source", "0/16B6C50", "tx-1", 3);

    let mut envelope = TransactionEnvelope::strict(StrictEnvelope {
        source_id: "source".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-1".to_string(),
        begin_lsn: "0/16B6B00".to_string(),
        commit_lsn: "0/16B6C50".to_string(),
        commit_timestamp_ms: 1_786_420_000_000,
        changes: vec![insert_change(), update],
    });
    envelope.schema_versions = vec![schema_version()];
    envelope.ddl_events = vec![DdlEvent::additive_column(
        "tx-1",
        2,
        relation(),
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN \"discount_code\" text;",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let statements =
        plan_envelope(&envelope.dml_replay_after_ddl_barrier()).expect("DML replay plan");

    assert_eq!(
        statements
            .iter()
            .map(|statement| statement.operation)
            .collect::<Vec<_>>(),
        vec!["insert", "update"]
    );
    assert_eq!(
        statements
            .iter()
            .map(|statement| statement.total_order)
            .collect::<Vec<_>>(),
        vec![1, 3]
    );
}
