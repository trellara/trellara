use std::path::Path;

use crate::{
    lake_fanin_mode, lake_materialization_boundary, DatasetMode, FleetLakeFaninReadiness,
    FleetLakeFaninStatus, LakePlanCheckStatus, LakePlanSummary, TrellaraConfig,
};

pub(crate) fn fleet_lake_fanin_readiness(
    config: &TrellaraConfig,
    path: &Path,
) -> FleetLakeFaninReadiness {
    let lake_plan = LakePlanSummary::from_config(config);
    let status = lake_fanin_status(config, &lake_plan);
    let config_path = path.display();

    FleetLakeFaninReadiness {
        status,
        fanin_mode: lake_fanin_mode(config).to_string(),
        epoch_visibility_boundary: lake_materialization_boundary(config).to_string(),
        iceberg_checkpoint_receipt_gate: iceberg_checkpoint_receipt_gate().to_string(),
        straggler_policy: lake_straggler_policy(config).to_string(),
        source_watermark_rollup: source_watermark_rollup(config).to_string(),
        proof_commands: vec![
            format!("trellara lake epoch --config {config_path}"),
            format!(
                "trellara lake fanin verify --config {config_path} --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json>"
            ),
            format!("trellara lake writer-plan --config {config_path}"),
            "cargo test -p trellara-lake partition_manifest_transaction_mismatch_fails_closed --lib && cargo test -p trellara-lake partition_manifest_duplicate_partition_id_fails_closed --lib && cargo test -p trellara-lake manifest_boundary_mismatch_blocks_epoch_completion --lib".to_string(),
            format!("trellara lake spark-template current-state --config {config_path} --table <schema.table> --epoch-id <epoch-id>"),
            format!("trellara lake spark-template dashboard --config {config_path} --table <schema.table> --epoch-id <epoch-id>"),
        ],
        guidance: lake_fanin_guidance(config, &lake_plan, status),
    }
}

fn lake_fanin_status(config: &TrellaraConfig, lake_plan: &LakePlanSummary) -> FleetLakeFaninStatus {
    if lake_plan.blocking_check_count > 0 {
        FleetLakeFaninStatus::Blocked
    } else if config.dataset.mode == DatasetMode::PartitionedScaleMode
        || lake_plan.warning_check_count > 0
    {
        FleetLakeFaninStatus::PublishableWithGaps
    } else {
        FleetLakeFaninStatus::Ready
    }
}

fn lake_straggler_policy(config: &TrellaraConfig) -> &'static str {
    match config.dataset.mode {
        DatasetMode::PartitionedScaleMode => {
            "wait_all_required by default; publish_with_gaps requires explicit missing-source acceptance and per-source gap evidence"
        }
        DatasetMode::StrictTransactionOrder => {
            "wait_for_complete_source_transaction before publishing each epoch"
        }
    }
}

fn iceberg_checkpoint_receipt_gate() -> &'static str {
    "epoch metadata is publishable only after every planned Iceberg table append has a matching checkpoint receipt; partial table receipts keep the epoch invisible"
}

fn source_watermark_rollup(config: &TrellaraConfig) -> &'static str {
    match config.dataset.mode {
        DatasetMode::PartitionedScaleMode => {
            "per-source and per-partition watermarks roll up to the global low watermark; lagging sources remain explicit epoch gaps"
        }
        DatasetMode::StrictTransactionOrder if config.dataset.strict_chunking.is_some() => {
            "strict chunk manifests roll up to one source commit watermark per epoch"
        }
        DatasetMode::StrictTransactionOrder => {
            "complete source transaction envelopes roll up to one source commit watermark per epoch"
        }
    }
}

fn lake_fanin_guidance(
    config: &TrellaraConfig,
    lake_plan: &LakePlanSummary,
    status: FleetLakeFaninStatus,
) -> Vec<String> {
    let mut guidance = Vec::new();
    match status {
        FleetLakeFaninStatus::Ready => guidance.push(
            "publish Spark-derived tables only after lake fan-in verification reports match and Iceberg checkpoint receipts prove every table append"
                .to_string(),
        ),
        FleetLakeFaninStatus::PublishableWithGaps => guidance.push(
            "publish epochs with gaps only after customer acceptance names the missing, lagging, or quarantined sources"
                .to_string(),
        ),
        FleetLakeFaninStatus::Blocked => guidance.push(
            "resolve blocked lake fan-in checks before writing epoch metadata".to_string(),
        ),
    }

    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        guidance.push(
            "run partition-watermarks before lake epoch publication so the manifest and global low watermark define visibility"
                .to_string(),
        );
    }

    for check in &lake_plan.checks {
        if check.status != LakePlanCheckStatus::Ready {
            guidance.push(check.message.clone());
        }
    }

    guidance
}
