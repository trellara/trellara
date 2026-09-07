use crate::{
    lake_visibility_boundary_message, DatasetMode, LakePlanCheck, LakePlanCheckStatus,
    LakeTablePlan, TrellaraConfig,
};

pub(crate) fn lake_plan_checks(
    config: &TrellaraConfig,
    tables: &[LakeTablePlan],
) -> Vec<LakePlanCheck> {
    let mut checks = vec![LakePlanCheck {
        name: "transaction_boundary".to_string(),
        status: LakePlanCheckStatus::Ready,
        message: lake_transaction_boundary_check_message(config),
        recommendation: Some(match config.dataset.mode {
            DatasetMode::StrictTransactionOrder => {
                if config.dataset.strict_chunking.is_some() {
                    "gate raw CDC and epoch metadata on the strict chunk manifest and commit marker before using the source commit LSN as the lake checkpoint".to_string()
                } else {
                    "use source commit LSN as the table checkpoint and Iceberg snapshot boundary"
                        .to_string()
                }
            }
            DatasetMode::PartitionedScaleMode => {
                "gate raw CDC and epoch metadata on partition-watermarks before exposing an epoch"
                    .to_string()
            }
        }),
    }];

    checks.push(lake_schema_fingerprint_check(tables));
    checks.push(lake_primary_key_check(tables));

    checks
}

pub(crate) fn lake_recommended_next_steps(
    blocking_check_count: usize,
    warning_check_count: usize,
) -> Vec<String> {
    let mut steps = Vec::new();
    if warning_check_count > 0 {
        steps.push(
            "trellara schema-discover --config <flow.yml> and pin schema fingerprints".to_string(),
        );
    }
    if blocking_check_count > 0 {
        steps.push("resolve blocked lake fan-in checks before writing epoch metadata".to_string());
    }
    steps.extend([
        "trellara partition-watermarks --config <flow.yml> for partitioned flows before exposing lake epochs".to_string(),
        "trellara lake fanin verify after the first raw CDC and epoch metadata rehearsal".to_string(),
    ]);
    steps
}

fn lake_schema_fingerprint_check(tables: &[LakeTablePlan]) -> LakePlanCheck {
    let missing_fingerprints: Vec<_> = tables
        .iter()
        .filter(|table| table.source_schema_fingerprint.is_none())
        .map(|table| table.relation.clone())
        .collect();
    if missing_fingerprints.is_empty() {
        LakePlanCheck {
            name: "schema_fingerprints".to_string(),
            status: LakePlanCheckStatus::Ready,
            message: "all lake tables have pinned source schema fingerprints".to_string(),
            recommendation: None,
        }
    } else {
        LakePlanCheck {
            name: "schema_fingerprints".to_string(),
            status: LakePlanCheckStatus::Warning,
            message: format!(
                "missing source schema fingerprints for {}",
                missing_fingerprints.join(", ")
            ),
            recommendation: Some(
                "run trellara schema-discover and pin dataset.tables[].contract.source_schema_fingerprint before enabling Iceberg writes".to_string(),
            ),
        }
    }
}

fn lake_primary_key_check(tables: &[LakeTablePlan]) -> LakePlanCheck {
    let missing_primary_keys: Vec<_> = tables
        .iter()
        .filter(|table| table.primary_key.trim().is_empty())
        .map(|table| table.relation.clone())
        .collect();
    if missing_primary_keys.is_empty() {
        LakePlanCheck {
            name: "primary_keys".to_string(),
            status: LakePlanCheckStatus::Ready,
            message: "all Spark current-state and SCD2 templates have configured primary keys"
                .to_string(),
            recommendation: None,
        }
    } else {
        LakePlanCheck {
            name: "primary_keys".to_string(),
            status: LakePlanCheckStatus::Warning,
            message: format!("missing primary keys for {}", missing_primary_keys.join(", ")),
            recommendation: Some(
                "set dataset.tables[].verify.primary_key before running Spark current-state or SCD2 templates; append-only raw CDC can still be written with Trellara idempotency keys"
                    .to_string(),
            ),
        }
    }
}

fn lake_transaction_boundary_check_message(config: &TrellaraConfig) -> String {
    match config.dataset.mode {
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "strict chunked mode must commit lake visibility only after every chunk, the manifest, and the commit marker prove the full source transaction is present".to_string()
        }
        DatasetMode::StrictTransactionOrder => {
            "strict mode can publish each Iceberg commit from a complete source transaction envelope"
                .to_string()
        }
        DatasetMode::PartitionedScaleMode => {
            "partitioned mode must commit Iceberg visibility only after the manifest and commit marker barrier plus global low watermark prove every partition chunk is present".to_string()
        }
    }
}

pub(crate) fn lake_materialization_boundary(config: &TrellaraConfig) -> String {
    lake_visibility_boundary_message(config).to_string()
}
