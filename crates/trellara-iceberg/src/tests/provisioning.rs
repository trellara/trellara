use super::*;

#[test]
fn provisioning_plan_declares_create_verify_and_ddl_gated_evolution() {
    let plans =
        plan_raw_cdc_iceberg_table_provisioning(&raw_cdc_plan(), &commit_config()).expect("plans");

    assert_eq!(plans.len(), 2);
    let orders = plans
        .iter()
        .find(|plan| plan.lake_table_name == "retail_raw_orders")
        .expect("orders plan");
    assert_eq!(
        orders.target.qualified_name(),
        "analytics.retail.orders_cdc"
    );
    assert_eq!(
        orders.actions,
        vec![
            IcebergTableProvisioningAction::EnsureNamespace,
            IcebergTableProvisioningAction::CreateTableIfMissing,
            IcebergTableProvisioningAction::VerifyCompatible,
            IcebergTableProvisioningAction::EvolveSchemaAfterDdlAck,
        ]
    );
    assert_eq!(
        orders.partition_fields,
        vec![
            IcebergRawCdcPartitionField::identity(26, "epoch_id"),
            IcebergRawCdcPartitionField::identity(2, "source_bucket"),
        ]
    );
    assert!(orders.schema_evolution_gate.contains("DDL acknowledgement"));
    assert!(orders.partition_evolution_gate.contains("pending-epoch"));
}

#[test]
fn metadata_specs_cover_supporting_evidence_and_commit_completeness_last() {
    let specs = plan_iceberg_metadata_table_specs(&raw_cdc_plan(), &commit_config())
        .expect("metadata specs");
    assert_eq!(specs.len(), 6);
    assert_eq!(
        specs.iter().map(|spec| spec.kind).collect::<Vec<_>>(),
        vec![
            IcebergMetadataTableKind::Source,
            IcebergMetadataTableKind::Table,
            IcebergMetadataTableKind::Partition,
            IcebergMetadataTableKind::Quarantine,
            IcebergMetadataTableKind::Verification,
            IcebergMetadataTableKind::Completeness,
        ]
    );
    assert!(specs
        .windows(2)
        .all(|pair| pair[0].commit_order < pair[1].commit_order));
    assert_eq!(
        specs.last().expect("completeness").target.qualified_name(),
        "analytics.retail._trellara_epochs"
    );
    assert!(specs
        .iter()
        .all(|spec| spec.schema_fingerprint_sha256.len() == 64));
}

#[cfg(feature = "iceberg-rust")]
#[tokio::test]
async fn production_provisioning_requires_matching_ack_and_is_idempotent() {
    use std::collections::HashMap;

    use apache_iceberg::memory::{MemoryCatalogBuilder, MEMORY_CATALOG_WAREHOUSE};
    use apache_iceberg::CatalogBuilder;
    use trellara_lake::{raw_cdc_lake_ddl_ack_evidence, RawCdcLakeDdlAckRequest};

    let unique = format!("trellara-iceberg-provision-{}", uuid::Uuid::new_v4());
    let warehouse = std::env::temp_dir().join(unique);
    let catalog = MemoryCatalogBuilder::default()
        .load(
            "test",
            HashMap::from([(
                MEMORY_CATALOG_WAREHOUSE.to_string(),
                format!("file://{}", warehouse.display()),
            )]),
        )
        .await
        .expect("memory catalog");
    let write_plan = raw_cdc_plan();
    let plans = plan_raw_cdc_iceberg_table_provisioning(&write_plan, &commit_config())
        .expect("provisioning plans");

    let error = provision_raw_cdc_iceberg_tables(&catalog, "retail", &plans, None)
        .await
        .expect_err("DDL without acknowledgement");
    assert!(matches!(
        error,
        IcebergIntegrationError::DdlAcknowledgementRequired { .. }
    ));

    let evidence = raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "database-a".to_string(),
        dataset_id: "retail".to_string(),
        barrier_id: "barrier-1".to_string(),
        ack_lsn: "0/16B6C50".to_string(),
        schema_version: "schema-v2".to_string(),
        epoch_id: write_plan.epoch_id.clone(),
        metadata_table: write_plan.epoch_metadata.epochs_table.clone(),
        partition_metadata_table: write_plan.epoch_metadata.epoch_partitions_table.clone(),
        manifest_digest: write_plan.epoch_metadata.epoch_row.manifest_digest.clone(),
    })
    .expect("lake ack evidence");
    let ack = IcebergDdlAcknowledgement::from_raw_cdc_lake_ack(
        &evidence,
        plans
            .iter()
            .map(|plan| plan.schema_fingerprint_sha256.clone()),
    )
    .expect("Iceberg DDL authorization");
    let created = provision_raw_cdc_iceberg_tables(&catalog, "retail", &plans, Some(&ack))
        .await
        .expect("create tables");
    assert!(created
        .iter()
        .all(|outcome| outcome.status == IcebergTableProvisioningStatus::Created));

    let replay = provision_raw_cdc_iceberg_tables(&catalog, "retail", &plans, None)
        .await
        .expect("compatible replay needs no mutation authorization");
    assert!(replay
        .iter()
        .all(|outcome| outcome.status == IcebergTableProvisioningStatus::Compatible));
    drop(catalog);
    let _ = std::fs::remove_dir_all(warehouse);
}
