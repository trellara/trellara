use super::*;
use trellara_lake::LakeCompletenessState;

#[test]
fn plans_deterministic_table_appends_with_lineage_properties() {
    let write_plan = raw_cdc_plan();
    let config = commit_config();
    let files = completed_files();

    let first = plan_iceberg_epoch_commit(&write_plan, &config, files.clone()).expect("plan");
    let second = plan_iceberg_epoch_commit(&write_plan, &config, files.into_iter().rev().collect())
        .expect("plan");

    assert_eq!(first, second);
    assert_eq!(first.tables.len(), 2);
    assert_eq!(first.file_count, 2);
    assert_eq!(first.record_count, 3);
    assert!(!first.accepted_complete_with_gaps);
    for table in &first.tables {
        assert_eq!(table.epoch_commit_id, first.epoch_commit_id);
        assert_eq!(table.commit_uuid.len(), 36);
        assert!(table.requirements.require_table_uuid_match);
        assert!(table.requirements.require_current_snapshot_match);
        assert!(table.requirements.check_duplicate_files);
        assert_eq!(
            table
                .snapshot_properties
                .get(SNAPSHOT_PROPERTY_MANIFEST_DIGEST),
            Some(&MANIFEST_DIGEST.to_string())
        );
        assert_eq!(
            table
                .snapshot_properties
                .get(SNAPSHOT_PROPERTY_EPOCH_COMMIT_ID),
            Some(&first.epoch_commit_id)
        );
    }
}

#[test]
fn rejects_missing_and_unplanned_completed_files() {
    let write_plan = raw_cdc_plan();
    let config = commit_config();
    let mut missing = completed_files();
    missing.pop();

    assert!(matches!(
        plan_iceberg_epoch_commit(&write_plan, &config, missing),
        Err(IcebergIntegrationError::MissingCompletedDataFile { .. })
    ));

    let mut extra = completed_files();
    let mut unplanned = extra[0].clone();
    unplanned.planned_object_key = "unplanned.parquet".to_string();
    unplanned.file_uri = "s3://lake/unplanned.parquet".to_string();
    extra.push(unplanned);
    assert!(matches!(
        plan_iceberg_epoch_commit(&write_plan, &config, extra),
        Err(IcebergIntegrationError::UnplannedCompletedDataFile { .. })
    ));
}

#[test]
fn rejects_file_evidence_that_does_not_match_the_write_plan() {
    let write_plan = raw_cdc_plan();
    let config = commit_config();
    let mut files = completed_files();
    files[0].record_count = 3;

    assert!(matches!(
        plan_iceberg_epoch_commit(&write_plan, &config, files),
        Err(IcebergIntegrationError::CompletedDataFileMismatch {
            field: "record_count",
            ..
        })
    ));
}

#[test]
fn rejects_invalid_file_uri_before_catalog_work() {
    let write_plan = raw_cdc_plan();
    let config = commit_config();
    let mut files = completed_files();
    files[0].file_uri = "relative/orders.parquet".to_string();

    assert!(matches!(
        plan_iceberg_epoch_commit(&write_plan, &config, files),
        Err(IcebergIntegrationError::InvalidCompletedDataFile {
            field: "file_uri",
            ..
        })
    ));
}

#[test]
fn complete_with_gaps_requires_explicit_acceptance() {
    let mut write_plan = raw_cdc_plan();
    write_plan.epoch_metadata.epoch_row.state = LakeCompletenessState::CompleteWithGaps;
    write_plan.epoch_metadata.epoch_row.policy = "publish_with_gaps".to_string();
    write_plan.epoch_metadata.epoch_row.complete_source_count = 1;
    write_plan.epoch_metadata.epoch_row.missing_source_count = 1;

    assert!(matches!(
        plan_iceberg_epoch_commit(&write_plan, &commit_config(), completed_files()),
        Err(IcebergIntegrationError::Lake(
            trellara_lake::LakeError::EpochRequiresGapAcceptance { .. }
        ))
    ));

    let accepted = plan_iceberg_epoch_commit(
        &write_plan,
        &commit_config().accepting_complete_with_gaps(),
        completed_files(),
    )
    .expect("explicit gap acceptance");
    assert!(accepted.accepted_complete_with_gaps);
}

#[test]
fn duplicate_mapping_and_target_are_rejected() {
    let mut duplicate_mapping = commit_config();
    duplicate_mapping
        .table_mappings
        .push(duplicate_mapping.table_mappings[0].clone());
    assert!(matches!(
        plan_iceberg_epoch_commit(&raw_cdc_plan(), &duplicate_mapping, completed_files()),
        Err(IcebergIntegrationError::DuplicateTableMapping { .. })
    ));

    let mut duplicate_target = commit_config();
    duplicate_target.table_mappings[1].target = duplicate_target.table_mappings[0].target.clone();
    assert!(matches!(
        plan_iceberg_epoch_commit(&raw_cdc_plan(), &duplicate_target, completed_files()),
        Err(IcebergIntegrationError::DuplicateIcebergTarget { .. })
    ));
}
