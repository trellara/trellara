#[path = "lake_fanin_completeness_evidence.rs"]
mod lake_fanin_completeness_evidence;
#[path = "lake_fanin_completeness_policy.rs"]
mod lake_fanin_completeness_policy;

use crate::{
    lake_fanin::read_lake_epoch_summary, LakeDdlSummary, LakeEpochSummary,
    LakeFaninCompletenessArgs, LakeFaninCompletenessSummary, LakeFaninVerifySummary,
    TrellaraConfig,
};
use lake_fanin_completeness_evidence::{
    ddl_table_refs, proof_artifacts, proof_commands, source_state_counts, spark_templates,
};
use lake_fanin_completeness_policy::{completeness_decision, deferred_sink_work, recovery_path};

impl LakeFaninCompletenessSummary {
    pub(crate) fn from_args(
        config: &TrellaraConfig,
        args: &LakeFaninCompletenessArgs,
    ) -> crate::Result<Self> {
        let stream_epoch = read_lake_epoch_summary(&args.stream_epoch)?;
        let lake_epoch = read_lake_epoch_summary(&args.lake_epoch)?;
        let verification = LakeFaninVerifySummary::from_epochs(
            config,
            args.config.as_path(),
            &args.stream_epoch,
            &args.lake_epoch,
            stream_epoch,
            lake_epoch.clone(),
            args.accept_complete_with_gaps,
        );

        Ok(Self::from_verification(
            config,
            args,
            lake_epoch,
            verification,
        ))
    }

    fn from_verification(
        config: &TrellaraConfig,
        args: &LakeFaninCompletenessArgs,
        lake_epoch: LakeEpochSummary,
        verification: LakeFaninVerifySummary,
    ) -> Self {
        let ddl = LakeDdlSummary::from_config(config);
        let source_state_counts = source_state_counts(&lake_epoch.source_watermarks);
        let decision = completeness_decision(
            verification.status,
            verification.spark_consumption_allowed,
            lake_epoch.state,
            args.accept_complete_with_gaps,
        );

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
            positioning:
                "prove queryable epoch completeness with append-only raw CDC and metadata gates; do not position this as a generic Iceberg sink"
                    .to_string(),
            epoch_id: lake_epoch.epoch_id.clone(),
            completeness_state: lake_epoch.state,
            decision,
            accepted_complete_with_gaps: verification.spark_consumption_allowed
                && args.accept_complete_with_gaps
                && lake_epoch.state == trellara_lake::LakeCompletenessState::CompleteWithGaps,
            spark_consumption_allowed: verification.spark_consumption_allowed,
            spark_consumption_contract: verification.spark_consumption_contract,
            spark_consumption_gate: verification.spark_consumption_gate.clone(),
            required_source_count: lake_epoch.required_source_count,
            complete_source_count: lake_epoch.complete_source_count,
            missing_source_count: lake_epoch.missing_source_count,
            quarantined_source_count: lake_epoch.quarantined_source_count,
            transaction_count: lake_epoch.transaction_count,
            change_count: lake_epoch.change_count,
            checksum_rollup: lake_epoch.checksum_rollup,
            manifest_digest: lake_epoch.manifest_digest.clone(),
            source_state_counts,
            source_rows: lake_epoch.source_watermarks,
            table_rollups: lake_epoch.table_rollups,
            quarantine_entries: lake_epoch.quarantine_entries,
            raw_cdc_tables: ddl_table_refs(&ddl, true),
            epoch_metadata_tables: ddl_table_refs(&ddl, false),
            spark_templates: spark_templates(
                args,
                &lake_epoch.epoch_id,
                &verification.spark_consumption_gate,
            ),
            proof_artifacts: proof_artifacts(args),
            proof_commands: proof_commands(args, &verification.epoch_id),
            recovery_path: recovery_path(decision, &verification),
            deferred_sink_work: deferred_sink_work(),
            verification,
        }
    }
}
