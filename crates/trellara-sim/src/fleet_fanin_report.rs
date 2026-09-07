use crate::fleet_fanin::{
    FleetFanInFailurePoint, FleetFanInPartitionRollup, FleetFanInQuarantineEntry,
    FleetFanInSimulationReport, FleetFanInSimulationStep, FleetFanInSourceWatermark,
    FleetFanInTableRollup,
};
use trellara_lake::{
    LakeCompletenessState, LakeEpoch, LakeEpochPartition, LakeEpochSource, LakeEpochSourceState,
    LakeEpochTable, LakeStragglerPolicy,
};

pub(crate) struct FleetFanInReportInput {
    pub(crate) seed: u64,
    pub(crate) failure_point: FleetFanInFailurePoint,
    pub(crate) epoch: LakeEpoch,
    pub(crate) recovered_from: Option<LakeCompletenessState>,
    pub(crate) duplicate_replay_count: usize,
    pub(crate) passed: bool,
    pub(crate) injected_failure: Option<String>,
    pub(crate) quarantine_entries: Vec<FleetFanInQuarantineEntry>,
    pub(crate) steps: Vec<FleetFanInSimulationStep>,
}

pub(crate) fn build_fleet_fanin_report(input: FleetFanInReportInput) -> FleetFanInSimulationReport {
    FleetFanInSimulationReport {
        seed: input.seed,
        failure_point: input.failure_point,
        epoch_id: input.epoch.epoch_id,
        dataset_id: input.epoch.dataset_id,
        required_source_count: input.epoch.required_source_count,
        complete_source_count: input.epoch.complete_source_count,
        missing_source_count: input.epoch.missing_source_count,
        quarantined_source_count: input.epoch.quarantined_source_count,
        transaction_count: input.epoch.transaction_count,
        change_count: input.epoch.change_count,
        duplicate_replay_count: input.duplicate_replay_count,
        straggler_policy: straggler_policy_label(&input.epoch.straggler_policy).to_string(),
        initial_state: input.recovered_from.unwrap_or(input.epoch.state),
        recovered_state: input.recovered_from.map(|_| input.epoch.state),
        verification_status: input.epoch.verification.status,
        passed: input.passed,
        injected_failure: input.injected_failure,
        source_watermarks: input.epoch.sources.iter().map(source_watermark).collect(),
        table_rollups: input.epoch.tables.iter().map(table_rollup).collect(),
        partition_rollups: input
            .epoch
            .partitions
            .iter()
            .map(partition_rollup)
            .collect(),
        quarantine_entries: input.quarantine_entries,
        steps: input.steps,
    }
}

fn straggler_policy_label(policy: &LakeStragglerPolicy) -> &'static str {
    match policy {
        LakeStragglerPolicy::WaitAllRequired => "wait_all_required",
        LakeStragglerPolicy::PublishWithGaps { .. } => "publish_with_gaps",
        LakeStragglerPolicy::QuarantineOnGap => "quarantine_on_gap",
    }
}

fn source_watermark(source: &LakeEpochSource) -> FleetFanInSourceWatermark {
    FleetFanInSourceWatermark {
        source_id: source.source_id.clone(),
        state: source_state_label(source.state).to_string(),
        start_lsn: source.start_lsn.clone(),
        end_lsn: source.end_lsn.clone(),
        transaction_count: source.transaction_count,
        change_count: source.change_count,
        gap_reason: source.gap_reason.clone(),
    }
}

fn table_rollup(table: &LakeEpochTable) -> FleetFanInTableRollup {
    FleetFanInTableRollup {
        relation: table.relation.clone(),
        transaction_count: table.transaction_count,
        change_count: table.change_count,
        checksum_rollup: table.checksum_rollup,
    }
}

fn partition_rollup(partition: &LakeEpochPartition) -> FleetFanInPartitionRollup {
    FleetFanInPartitionRollup {
        source_id: partition.source_id.clone(),
        partition_id: partition.partition_id,
        first_commit_lsn: partition.first_commit_lsn.clone(),
        last_commit_lsn: partition.last_commit_lsn.clone(),
        transaction_count: partition.transaction_count,
        event_count: partition.event_count,
        checksum_rollup: partition.checksum_rollup,
    }
}

fn source_state_label(state: LakeEpochSourceState) -> &'static str {
    match state {
        LakeEpochSourceState::Complete => "complete",
        LakeEpochSourceState::Lagging => "lagging",
        LakeEpochSourceState::Missing => "missing",
        LakeEpochSourceState::Quarantined => "quarantined",
        LakeEpochSourceState::Reseeding => "reseeding",
    }
}

#[cfg(test)]
#[path = "fleet_fanin_report_tests.rs"]
mod fleet_fanin_report_tests;
