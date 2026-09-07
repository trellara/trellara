use super::*;

#[test]
fn production_writer_contract_locks_durable_visibility_order() {
    let contract = IcebergWriterContract::production_v1();

    assert_eq!(contract.catalog, IcebergCatalogContract::Rest);
    assert_eq!(
        contract.object_store,
        IcebergObjectStoreContract::S3Compatible
    );
    assert_eq!(contract.write_mode, IcebergWriteMode::AppendOnlyRawCdc);
    assert!(contract.require_durable_upload_proof);
    assert!(contract.require_immutable_object_write);
    assert!(contract.persist_intent_before_catalog_call);
    assert!(contract.validate_snapshot_before_receipt);
    assert!(contract.publish_epoch_metadata_after_all_receipts);
    assert!(contract.publish_queryable_epoch_completeness_table);
    assert!(contract.validate().is_ok());
}

#[test]
fn production_writer_contract_rejects_overwritable_objects() {
    let mut contract = IcebergWriterContract::production_v1();
    contract.require_immutable_object_write = false;

    assert!(matches!(
        contract.validate(),
        Err(IcebergIntegrationError::InvalidWriterContract {
            field: "require_immutable_object_write",
            ..
        })
    ));
}

#[test]
fn production_writer_contract_rejects_early_epoch_visibility() {
    let mut contract = IcebergWriterContract::production_v1();
    contract.publish_epoch_metadata_after_all_receipts = false;

    assert!(matches!(
        contract.validate(),
        Err(IcebergIntegrationError::InvalidWriterContract {
            field: "publish_epoch_metadata_after_all_receipts",
            ..
        })
    ));
}

#[test]
fn production_writer_contract_rejects_missing_queryable_completeness_table() {
    let mut contract = IcebergWriterContract::production_v1();
    contract.publish_queryable_epoch_completeness_table = false;

    assert!(matches!(
        contract.validate(),
        Err(IcebergIntegrationError::InvalidWriterContract {
            field: "publish_queryable_epoch_completeness_table",
            ..
        })
    ));
}

#[test]
fn completed_file_requires_cryptographic_content_identity() {
    let raw_plan = raw_cdc_plan();
    let mut files = completed_files();
    files[0].content_sha256 = "not-a-sha256".to_string();

    assert!(matches!(
        plan_iceberg_epoch_commit(&raw_plan, &commit_config(), files),
        Err(IcebergIntegrationError::InvalidCompletedDataFile {
            field: "content_sha256",
            ..
        })
    ));
}
