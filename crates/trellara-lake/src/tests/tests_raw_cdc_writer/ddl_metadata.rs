use super::*;

#[test]
fn raw_cdc_epoch_writer_rejects_ddl_without_schema_version_evidence() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        2,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::additive_column(
        "tx-1",
        1,
        relation(),
        "alter table public.sales add column discount_code text",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("missing DDL schema version evidence");

    assert!(matches!(
        error,
        LakeError::MissingRawCdcDdlSchemaVersion { relation } if relation == "public.sales"
    ));
}

#[test]
fn raw_cdc_epoch_writer_rejects_ddl_schema_version_mismatch() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        2,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    envelope.schema_versions = vec![trellara_protocol::RelationSchemaVersion {
        relation: Some(relation()),
        version: 12_345,
    }];
    envelope.ddl_events = vec![trellara_protocol::DdlEvent::additive_column(
        "tx-1",
        1,
        relation(),
        "alter table public.sales add column discount_code text",
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let error = plan_raw_cdc_epoch_writes(&raw_writer_config(4), &config(), &[envelope])
        .expect_err("mismatched DDL schema version evidence");

    assert!(matches!(
        error,
        LakeError::RawCdcDdlSchemaVersionMismatch { relation, expected: 67_890, actual: 12_345 }
            if relation == "public.sales"
    ));
}
