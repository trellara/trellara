use trellara_sim::{run_fleet_fanin_simulation, FleetFanInSimulationConfig};

use crate::{
    lake_epoch_customer_decision, lake_epoch_manifest_digest, lake_epoch_partition_rollups,
    lake_epoch_partition_skew, lake_epoch_quarantine_entries, lake_epoch_recommended_next_steps,
    lake_epoch_source_watermarks, lake_epoch_straggler_policy_decision, lake_epoch_table_rollups,
    lake_epoch_watermark_rollup, lake_fanin_mode, lake_visibility_boundary_message, LakeEpochArgs,
    LakeEpochManifestDigestInput, LakeEpochSummary, TrellaraConfig,
};

impl LakeEpochSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, args: &LakeEpochArgs) -> Self {
        let mut simulation_config =
            FleetFanInSimulationConfig::new(20_260_816, args.scenario.failure_point())
                .with_dataset_id(config.dataset.id.clone());
        simulation_config.store_count = args.required_source_count.max(args.offline_source_count);
        simulation_config.offline_store_count = args.offline_source_count;
        simulation_config.duplicate_replay_count = args.duplicate_replay_count;
        let report = run_fleet_fanin_simulation(simulation_config);
        let state = report.recovered_state.unwrap_or(report.initial_state);
        let source_watermarks = lake_epoch_source_watermarks(&report.source_watermarks);
        let table_rollups = lake_epoch_table_rollups(&report.table_rollups);
        let partition_rollups = lake_epoch_partition_rollups(&report.partition_rollups);
        let partition_skew = lake_epoch_partition_skew(&partition_rollups);
        let checksum_rollup = table_rollups
            .iter()
            .fold(0u64, |rollup, table| rollup ^ table.checksum_rollup);
        let quarantine_entries = lake_epoch_quarantine_entries(&report.quarantine_entries);
        let watermark_rollup = lake_epoch_watermark_rollup(&source_watermarks);
        let manifest_digest = lake_epoch_manifest_digest(
            &LakeEpochManifestDigestInput {
                epoch_id: &report.epoch_id,
                dataset_id: &config.dataset.id,
                state,
                straggler_policy: &report.straggler_policy,
                required_source_count: report.required_source_count,
                complete_source_count: report.complete_source_count,
                missing_source_count: report.missing_source_count,
                quarantined_source_count: report.quarantined_source_count,
                transaction_count: report.transaction_count,
                change_count: report.change_count,
            },
            &source_watermarks,
            &table_rollups,
            &partition_rollups,
            &partition_skew,
            &quarantine_entries,
        );
        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            mode: config.dataset.mode.to_string(),
            contract: "fleet_fanin_append_only_raw_cdc_with_epoch_completeness".to_string(),
            fanin_mode: lake_fanin_mode(config).to_string(),
            scenario: args.scenario,
            epoch_id: report.epoch_id,
            state,
            recovered_state: report.recovered_state,
            required_source_count: report.required_source_count,
            complete_source_count: report.complete_source_count,
            missing_source_count: report.missing_source_count,
            quarantined_source_count: report.quarantined_source_count,
            transaction_count: report.transaction_count,
            change_count: report.change_count,
            checksum_rollup,
            duplicate_replay_count: report.duplicate_replay_count,
            straggler_policy: report.straggler_policy.clone(),
            straggler_policy_decision: lake_epoch_straggler_policy_decision(
                &report.straggler_policy,
                args.scenario,
            ),
            manifest_digest,
            watermark_rollup,
            source_watermarks,
            table_rollups,
            partition_rollups,
            partition_skew,
            quarantine_entries,
            verification_status: report.verification_status,
            passed: report.passed,
            injected_failure: report.injected_failure,
            visibility_boundary: lake_visibility_boundary_message(config).to_string(),
            customer_decision: lake_epoch_customer_decision(state).to_string(),
            proof_command: format!("cargo test -p trellara-sim {}", args.scenario.test_name()),
            recommended_next_steps: lake_epoch_recommended_next_steps(state),
        }
    }
}
