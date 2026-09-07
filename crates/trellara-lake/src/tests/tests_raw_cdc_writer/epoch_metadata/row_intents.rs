use super::*;

#[test]
fn raw_cdc_row_intents_include_pinned_source_schema_fingerprint() {
    let envelope = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )],
    );
    let config = LakePlanConfig::new(vec![
        LakeTableConfig::new("public", "sales", "id").with_source_schema_fingerprint(Some(42))
    ]);

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config, &[envelope])
        .expect("raw CDC writer plan");

    assert_eq!(plan.row_intents[0].schema_fingerprint, Some(42));
}

#[test]
fn raw_cdc_row_intents_include_schema_ddl_barrier_evidence() {
    let mut envelope = envelope(vec![change(
        Operation::Insert,
        2,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    envelope.schema_versions = vec![trellara_protocol::RelationSchemaVersion {
        relation: Some(relation()),
        version: 67_890,
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

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config(), &[envelope])
        .expect("raw CDC writer plan");

    let row = &plan.row_intents[0];
    assert_eq!(row.schema_version, Some(67_890));
    assert_eq!(
        row.ddl_barrier_id.as_deref(),
        Some("source-a:retail:tx-1:0/16B6C50:ddl")
    );
    assert_eq!(
        row.ddl_release_gate.as_deref(),
        Some(trellara_protocol::POST_DDL_DML_RELEASE_GATE)
    );
    assert_eq!(row.ddl_schema_fingerprint_before, Some(12_345));
    assert_eq!(row.ddl_schema_fingerprint_after, Some(67_890));
}

#[test]
fn raw_cdc_row_intents_include_partition_key() {
    let envelope = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(RowImage::new(vec![
                ColumnValue::text("id", 23, "sale-1", true),
                ColumnValue::text("store_id", 25, "store-001", false),
                ColumnValue::text("amount", 25, "10", false),
            ])),
        )],
    );
    let config = LakePlanConfig::new(vec![LakeTableConfig::new("public", "sales", "id")
        .with_partition_key_column(Some("store_id".to_string()))]);

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config, &[envelope])
        .expect("raw CDC writer plan");

    assert_eq!(
        plan.row_intents[0].partition_key.as_deref(),
        Some("store-001")
    );
}

#[test]
fn raw_cdc_row_intents_include_source_bucket() {
    let envelope = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![change(
            Operation::Insert,
            1,
            None,
            Some(row("sale-1", "10", "2026-01-01")),
        )],
    );

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(8), &config(), &[envelope])
        .expect("raw CDC writer plan");

    assert_eq!(
        plan.row_intents[0].source_bucket,
        plan.data_files[0].source_bucket
    );
}

#[test]
fn raw_cdc_row_intents_include_partition_manifest_boundary_evidence() {
    let mut envelope = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![
            change(
                Operation::Insert,
                1,
                None,
                Some(row("sale-1", "10", "2026-01-01")),
            ),
            change(
                Operation::Insert,
                2,
                None,
                Some(row("sale-2", "20", "2026-01-01")),
            ),
        ],
    );
    envelope.manifest = Some(trellara_protocol::TransactionManifest {
        transaction_id: "tx-1".to_string(),
        source_commit_lsn: "0/16B6C50".to_string(),
        source_commit_timestamp_ms: 1_700_000_000,
        global_event_count: 2,
        partitions: vec![
            ManifestPartition {
                id: 0,
                event_count: 1,
                first_total_order: 1,
                last_total_order: 1,
                checksum: 99,
            },
            ManifestPartition {
                id: 4,
                event_count: 1,
                first_total_order: 2,
                last_total_order: 2,
                checksum: 100,
            },
        ],
        affected_tables: affected_tables(2),
        boundary_mode: trellara_protocol::ManifestBoundaryMode::PartitionedScale as i32,
    });
    envelope.finalize_checksum();

    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config(), &[envelope])
        .expect("raw CDC writer plan");

    assert_eq!(plan.row_intents.len(), 2);
    assert!(plan.row_intents.iter().all(|row| {
        row.manifest_id.as_deref() == Some("tx-1:0/16B6C50:2")
            && row.manifest_boundary_mode.as_deref() == Some("partitioned_scale_mode")
            && row.manifest_global_event_count == Some(2)
            && row.manifest_participating_partition_count == Some(2)
    }));
    assert_eq!(plan.epoch_metadata.partition_rows.len(), 2);
    assert!(plan.epoch_metadata.partition_rows.iter().any(|row| {
        row.source_id == "store-001"
            && row.partition_id == 0
            && row.first_commit_lsn == "0/16B6C50"
            && row.last_commit_lsn == "0/16B6C50"
            && row.transaction_count == 1
            && row.event_count == 1
            && row.checksum_rollup == 99
    }));
    assert!(plan.epoch_metadata.partition_rows.iter().any(|row| {
        row.source_id == "store-001"
            && row.partition_id == 4
            && row.transaction_count == 1
            && row.event_count == 1
            && row.checksum_rollup == 100
    }));
}
