use super::*;

#[test]
fn checkpoint_preflight_records_missing_intents_before_commit() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");

    let decisions =
        plan_iceberg_checkpoint_preflight(&plan, &[], &[], "planned-at").expect("preflight");

    assert_eq!(decisions.len(), plan.tables.len());
    assert!(decisions.iter().all(|decision| {
        decision.action == IcebergPreflightAction::RecordIntentThenCommit
            && decision.reason.contains("no checkpoint intent exists")
    }));
}

#[test]
fn checkpoint_preflight_retries_after_recorded_intent_without_receipt() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let intents = iceberg_commit_intents_from_plan(&plan, "planned-at");

    let decisions =
        plan_iceberg_checkpoint_preflight(&plan, &intents, &[], "planned-at").expect("preflight");

    assert!(decisions.iter().all(|decision| {
        decision.action == IcebergPreflightAction::CommitAfterRecordedIntent
            && decision.reason.contains("no validated receipt")
    }));
}

#[test]
fn checkpoint_preflight_skips_tables_with_matching_receipts() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let receipts = plan
        .tables
        .iter()
        .enumerate()
        .map(|(index, table)| {
            let receipt = receipt(
                table,
                i64::try_from(index + 1).expect("snapshot id"),
                IcebergTableCommitStatus::Committed,
            );
            iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at")
                .expect("receipt row")
        })
        .collect::<Vec<_>>();

    let decisions =
        plan_iceberg_checkpoint_preflight(&plan, &[], &receipts, "planned-at").expect("preflight");

    assert!(decisions.iter().all(|decision| {
        decision.action == IcebergPreflightAction::SkipAlreadyCommitted
            && decision.reason.contains("already reached the catalog")
    }));
}

#[test]
fn checkpoint_preflight_rejects_mismatched_persisted_intent() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let mut intents = iceberg_commit_intents_from_plan(&plan, "planned-at");
    intents[0].record_count += 1;

    let error = plan_iceberg_checkpoint_preflight(&plan, &intents, &[], "planned-at")
        .expect_err("intent mismatch");

    assert!(matches!(
        error,
        IcebergIntegrationError::ConflictingCatalogEvidence { .. }
    ));
}

#[test]
fn checkpoint_preflight_rejects_mismatched_persisted_receipt() {
    let plan = plan_iceberg_epoch_commit(&raw_cdc_plan(), &commit_config(), completed_files())
        .expect("plan");
    let receipt = receipt(&plan.tables[0], 101, IcebergTableCommitStatus::Committed);
    let mut receipt_row =
        iceberg_commit_receipt_checkpoint_row(&plan, &receipt, "committed-at").expect("receipt");
    receipt_row.record_count += 1;

    let error = plan_iceberg_checkpoint_preflight(&plan, &[], &[receipt_row], "planned-at")
        .expect_err("receipt mismatch");

    assert!(matches!(
        error,
        IcebergIntegrationError::CommitReceiptMismatch {
            field: "record_count",
            ..
        }
    ));
}
