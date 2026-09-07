use super::*;

#[test]
fn plans_binary_values_as_bytea_parameters() {
    let mut change = insert_change();
    change.after = Some(row(vec![ColumnValue::binary(
        "payload",
        17,
        vec![1, 2, 3],
        false,
    )]));

    let statement = plan_change(&change).expect("binary insert plan");

    assert_eq!(
        statement.sql,
        "insert into \"public\".\"sales\" (\"payload\") values ($1)"
    );
    assert_eq!(
        statement.values,
        vec![SqlValue {
            column: "payload".to_string(),
            value: SqlValueData::Binary(vec![1, 2, 3]),
        }]
    );
}

#[test]
fn quarantine_reason_names_failure_class() {
    assert_eq!(
        quarantine_reason(&ApplyError::MissingKeyColumns { total_order: 7 }),
        "missing_key_columns"
    );
    assert_eq!(
        quarantine_reason(&ApplyError::UnchangedToastKeyColumn {
            total_order: 7,
            column: "id".to_string(),
        }),
        "unchanged_toast_key_column"
    );
    assert_eq!(
        quarantine_reason(&ApplyError::UnsupportedOperation(99)),
        "unsupported_operation"
    );
    assert_eq!(
        quarantine_reason(&ApplyError::NoRowsMatched {
            total_order: 2,
            operation: "update",
        }),
        "no_rows_matched"
    );
    assert_eq!(
        quarantine_reason(&ApplyError::DdlBarrierRequired {
            transaction_id: "tx-ddl".to_string(),
            boundary_kind: TransactionBoundaryKind::DdlOnly,
            partitioned_scale_decision: PartitionedScaleDecision::DdlBarrierRequired,
            reason: "DDL-only transaction must use the DDL barrier path".to_string(),
        }),
        "ddl_barrier_required"
    );
}

#[test]
fn quarantine_record_preserves_valid_transaction_boundary() {
    let apply_error = ApplyError::MissingKeyColumns { total_order: 7 };
    let record = crate::checkpoint_quarantine::quarantined_envelope_record(
        &envelope("tx-1", "0/16B6C50"),
        &apply_error,
    )
    .expect("quarantine record");

    assert_eq!(record.transaction_key.transaction_id, "tx-1");
    assert_eq!(record.transaction_key.commit_lsn, "0/16B6C50");
    assert_eq!(record.reason, "missing_key_columns");
    assert!(record.detail.contains("change 7"));
    assert!(record.detail.contains("key columns"));
}

#[test]
fn quarantine_record_canonicalizes_transaction_boundary_lsn() {
    let apply_error = ApplyError::MissingKeyColumns { total_order: 7 };
    let record = crate::checkpoint_quarantine::quarantined_envelope_record(
        &envelope("tx-1", "00000000/016B6C50"),
        &apply_error,
    )
    .expect("quarantine record");

    assert_eq!(record.transaction_key.commit_lsn, "0/16B6C50");
}

#[test]
fn quarantine_record_rejects_invalid_transaction_boundary() {
    let apply_error = ApplyError::MissingKeyColumns { total_order: 7 };
    let error = crate::checkpoint_quarantine::quarantined_envelope_record(
        &envelope("tx-1", "0/0"),
        &apply_error,
    )
    .expect_err("zero commit lsn rejected");

    assert!(error.to_string().contains("commit_lsn"));
    assert!(error.to_string().contains("LSN must be greater than zero"));
}

#[test]
fn identifiers_are_quoted() {
    let change = ChangeRecord {
        relation: Some(RelationId::new(42, "odd\"schema", "odd\"table")),
        ..insert_change()
    };

    let statement = plan_change(&change).expect("quoted insert");

    assert!(statement
        .sql
        .starts_with("insert into \"odd\"\"schema\".\"odd\"\"table\""));
}
