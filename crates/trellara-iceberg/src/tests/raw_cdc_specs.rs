use super::*;

#[test]
fn raw_cdc_table_specs_are_deterministic_and_partitioned_for_fleet_fan_in() {
    let write_plan = raw_cdc_plan();

    let specs =
        plan_raw_cdc_iceberg_table_specs(&write_plan, &commit_config()).expect("table specs");
    let repeat =
        plan_raw_cdc_iceberg_table_specs(&write_plan, &commit_config()).expect("repeat specs");

    assert_eq!(specs, repeat);
    assert_eq!(specs.len(), 2);
    let orders = specs
        .iter()
        .find(|spec| spec.lake_table_name == "retail_raw_orders")
        .expect("orders spec");
    assert_eq!(orders.relation, "public.orders");
    assert_eq!(
        orders.target.qualified_name(),
        "analytics.retail.orders_cdc"
    );
    assert_eq!(orders.schema_fingerprint_sha256.len(), 64);
    assert!(orders
        .columns
        .iter()
        .any(|column| column.name == "idempotency_key" && column.required));
    assert_eq!(
        orders.partition_fields,
        vec![
            IcebergRawCdcPartitionField::identity(26, "epoch_id"),
            IcebergRawCdcPartitionField::identity(2, "source_bucket"),
        ]
    );
    assert!(orders
        .columns
        .iter()
        .any(|column| column.name == "ddl_schema_fingerprint_before" && !column.required));
    assert!(orders
        .columns
        .iter()
        .any(|column| column.name == "ddl_schema_fingerprint_after" && !column.required));
    assert_eq!(orders.source_buckets, vec![0]);
}

#[test]
fn raw_cdc_table_specs_reject_missing_table_mapping() {
    let config = IcebergCommitConfig::new(vec![IcebergTableMapping::new(
        "retail_raw_orders",
        IcebergTableIdentifier::new(["analytics", "retail"], "orders_cdc").expect("target"),
    )]);

    let error = plan_raw_cdc_iceberg_table_specs(&raw_cdc_plan(), &config).expect_err("mapping");

    assert!(matches!(
        error,
        IcebergIntegrationError::MissingTableMapping {
            lake_table_name
        } if lake_table_name == "retail_raw_customers"
    ));
}
