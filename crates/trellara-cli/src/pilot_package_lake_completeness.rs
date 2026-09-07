#[path = "pilot_package_lake_spark_templates.rs"]
mod pilot_package_lake_spark_templates;
#[path = "pilot_package_lake_completeness/types.rs"]
mod types;

use pilot_package_lake_spark_templates::spark_templates;
pub(crate) use types::*;

use crate::{
    lake_completeness_state_label, lake_epoch_source_state_label, LakeFaninVerifyStatus,
    PilotPackageMaterials,
};

impl LakeCompletenessEvidence {
    pub(crate) fn from_materials(materials: &PilotPackageMaterials) -> Self {
        Self {
            contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
            epoch_id: materials.lake_epoch.epoch_id.clone(),
            dataset_id: materials.lake_epoch.dataset_id.clone(),
            completeness_state: lake_completeness_state_label(materials.lake_epoch.state)
                .to_string(),
            verification_status: verify_status_label(materials.lake_verify.status).to_string(),
            spark_consumption_allowed: materials.lake_verify.spark_consumption_allowed,
            spark_consumption_contract: materials.lake_verify.spark_consumption_contract,
            spark_consumption_gate: materials.lake_verify.spark_consumption_gate.clone(),
            source_watermark_count: materials.lake_epoch.source_watermarks.len(),
            table_rollup_count: materials.lake_epoch.table_rollups.len(),
            partition_rollup_count: materials.lake_epoch.partition_rollups.len(),
            partition_skew: materials.lake_epoch.partition_skew.clone(),
            source_counts_match: materials.lake_verify.source_counts_match,
            stream_required_source_count: materials.lake_verify.stream_required_source_count,
            stream_complete_source_count: materials.lake_verify.stream_complete_source_count,
            stream_missing_source_count: materials.lake_verify.stream_missing_source_count,
            stream_quarantined_source_count: materials
                .lake_verify
                .stream_quarantined_source_count,
            lake_required_source_count: materials.lake_verify.lake_required_source_count,
            lake_complete_source_count: materials.lake_verify.lake_complete_source_count,
            lake_missing_source_count: materials.lake_verify.lake_missing_source_count,
            lake_quarantined_source_count: materials.lake_verify.lake_quarantined_source_count,
            required_source_count: materials.lake_epoch.required_source_count,
            complete_source_count: materials.lake_epoch.complete_source_count,
            missing_source_count: materials.lake_epoch.missing_source_count,
            quarantined_source_count: materials.lake_epoch.quarantined_source_count,
            transaction_count: materials.lake_epoch.transaction_count,
            change_count: materials.lake_epoch.change_count,
            checksum_rollup_match: materials.lake_verify.checksum_rollup_match,
            stream_checksum_rollup: materials.lake_verify.stream_checksum_rollup,
            lake_checksum_rollup: materials.lake_verify.lake_checksum_rollup,
            checksum_rollup: materials
                .lake_epoch
                .table_rollups
                .iter()
                .fold(0, |rollup, table| rollup ^ table.checksum_rollup),
            source_rows: source_evidence(materials),
            committer_strategy: materials.lake_writer_plan.committer_topology.strategy.clone(),
            source_ack_boundary: materials
                .lake_writer_plan
                .committer_topology
                .source_ack_boundary
                .clone(),
            catalog_backpressure_rule: materials
                .lake_writer_plan
                .committer_topology
                .catalog_backpressure_rule
                .clone(),
            recovery_scenario_codes: materials
                .lake_writer_plan
                .recovery_scenarios
                .iter()
                .map(|scenario| scenario.code.clone())
                .collect(),
            recovery_guidance: trellara_lake::lake_epoch_recovery_guidance(
                materials.lake_epoch.state,
            ),
            spark_templates: spark_templates(&materials.lake_verify.spark_consumption_gate),
            review_artifacts: vec![
                "lake-ddl.json".to_string(),
                "lake-epoch.json".to_string(),
                "lake-verify.json".to_string(),
                "lake-writer-plan.json".to_string(),
                "lake-fanin-run.json".to_string(),
                "spark-current-state.sql".to_string(),
                "spark-scd2.sql".to_string(),
                "spark-maintenance.sql".to_string(),
                "spark-completeness-dashboard.sql".to_string(),
            ],
            decision_rule:
                "Spark jobs may consume only verified complete epochs, or complete_with_gaps epochs when the caller explicitly accepts gaps"
                    .to_string(),
        }
    }
}

fn source_evidence(materials: &PilotPackageMaterials) -> Vec<LakeCompletenessSourceEvidence> {
    materials
        .lake_writer_plan
        .epoch_metadata
        .source_rows
        .iter()
        .map(|source| LakeCompletenessSourceEvidence {
            source_id: source.source_id.clone(),
            state: lake_epoch_source_state_label(source.state).to_string(),
            start_lsn: source.start_lsn.clone(),
            end_lsn: source.end_lsn.clone(),
            transaction_count: source.transaction_count,
            change_count: source.change_count,
            checksum_rollup: source.checksum_rollup,
            lag_reason: source.lag_reason.clone(),
        })
        .collect()
}

fn verify_status_label(status: LakeFaninVerifyStatus) -> &'static str {
    match status {
        LakeFaninVerifyStatus::Match => "match",
        LakeFaninVerifyStatus::Mismatch => "mismatch",
        LakeFaninVerifyStatus::Blocked => "blocked",
    }
}
