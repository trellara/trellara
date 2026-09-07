use serde::Serialize;
use trellara_sim::{
    FailurePoint, FleetFanInFailurePoint, QualificationFailurePoint, SnapshotFailurePoint,
    StrictChunkFailurePoint,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosRunSummary {
    pub(crate) metadata: ChaosReportMetadata,
    pub(crate) mode: String,
    pub(crate) passed: bool,
    pub(crate) scenario_count: usize,
    pub(crate) covered_scenarios: usize,
    pub(crate) invariant_count: usize,
    pub(crate) deterministic_seed: u64,
    pub(crate) simulation_count: usize,
    pub(crate) simulations_passed: bool,
    pub(crate) simulations: Vec<ChaosSimulationSummary>,
    pub(crate) snapshot_simulations: Vec<ChaosSnapshotSimulationSummary>,
    pub(crate) strict_chunk_simulations: Vec<ChaosStrictChunkSimulationSummary>,
    pub(crate) fleet_fanin_simulations: Vec<ChaosFleetFanInSimulationSummary>,
    pub(crate) qualification_simulations: Vec<ChaosQualificationSimulationSummary>,
    pub(crate) scenarios: Vec<ChaosScenarioSummary>,
    pub(crate) performance_envelope: ChaosPerformanceEnvelope,
    pub(crate) enterprise_review_gates: Vec<ChaosEnterpriseReviewGate>,
    pub(crate) verification_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosReportMetadata {
    pub(crate) artifact: String,
    pub(crate) report_version: String,
    pub(crate) package_version: String,
    pub(crate) source_revision: String,
    pub(crate) source_repository: String,
    pub(crate) workflow_run_url: String,
    pub(crate) generated_by: String,
    pub(crate) freshness_check: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosPerformanceEnvelope {
    pub(crate) quickstart_estimated_minutes: u32,
    pub(crate) quickstart_time_budget_minutes: u32,
    pub(crate) default_relay_max_transactions: u64,
    pub(crate) default_apply_max_messages: u64,
    pub(crate) stream_spill_threshold_changes: usize,
    pub(crate) stream_spill_location: String,
    pub(crate) bounded_large_transaction_mode: String,
    pub(crate) local_transport_ack: String,
    pub(crate) local_transport_replay: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosEnterpriseReviewGate {
    pub(crate) code: String,
    pub(crate) question: String,
    pub(crate) proof_surface: String,
    pub(crate) gate: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosSimulationSummary {
    pub(crate) seed: u64,
    pub(crate) failure_point: FailurePoint,
    pub(crate) passed: bool,
    pub(crate) transaction_count: usize,
    pub(crate) applied_transactions: usize,
    pub(crate) skipped_duplicates: usize,
    pub(crate) source_acknowledged_lsn: Option<u64>,
    pub(crate) relay_durable_lsn: Option<u64>,
    pub(crate) target_applied_lsn: Option<u64>,
    pub(crate) injected_failure: Option<String>,
    pub(crate) repro_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosSnapshotSimulationSummary {
    pub(crate) seed: u64,
    pub(crate) failure_point: SnapshotFailurePoint,
    pub(crate) passed: bool,
    pub(crate) table_count: usize,
    pub(crate) copied_tables: usize,
    pub(crate) writes_after_snapshot: usize,
    pub(crate) stream_replayed_transactions: usize,
    pub(crate) verification_matched: bool,
    pub(crate) contract_refreshed: bool,
    pub(crate) injected_failure: Option<String>,
    pub(crate) repro_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosStrictChunkSimulationSummary {
    pub(crate) seed: u64,
    pub(crate) failure_point: StrictChunkFailurePoint,
    pub(crate) passed: bool,
    pub(crate) chunk_count: usize,
    pub(crate) chunks_published: usize,
    pub(crate) duplicate_chunks: usize,
    pub(crate) manifest_published: bool,
    pub(crate) applied_transactions: usize,
    pub(crate) source_acknowledged_lsn: Option<u64>,
    pub(crate) target_applied_lsn: Option<u64>,
    pub(crate) injected_failure: Option<String>,
    pub(crate) repro_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosFleetFanInSimulationSummary {
    pub(crate) seed: u64,
    pub(crate) failure_point: FleetFanInFailurePoint,
    pub(crate) passed: bool,
    pub(crate) epoch_id: String,
    pub(crate) required_source_count: usize,
    pub(crate) complete_source_count: usize,
    pub(crate) missing_source_count: usize,
    pub(crate) quarantined_source_count: usize,
    pub(crate) transaction_count: usize,
    pub(crate) change_count: usize,
    pub(crate) duplicate_replay_count: usize,
    pub(crate) initial_state: trellara_lake::LakeCompletenessState,
    pub(crate) recovered_state: Option<trellara_lake::LakeCompletenessState>,
    pub(crate) verification_status: trellara_lake::LakeEpochVerificationStatus,
    pub(crate) injected_failure: Option<String>,
    pub(crate) repro_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosQualificationSimulationSummary {
    pub(crate) seed: u64,
    pub(crate) failure_point: QualificationFailurePoint,
    pub(crate) durable_boundary: String,
    pub(crate) invariant: String,
    pub(crate) passed: bool,
    pub(crate) transaction_count: usize,
    pub(crate) applied_transactions: usize,
    pub(crate) duplicate_replays: usize,
    pub(crate) source_acknowledged_lsn: Option<u64>,
    pub(crate) durable_lsn: Option<u64>,
    pub(crate) target_applied_lsn: Option<u64>,
    pub(crate) soak_hours: u32,
    pub(crate) large_transaction_change_count: usize,
    pub(crate) peak_memory_mib: u32,
    pub(crate) memory_ceiling_mib: u32,
    pub(crate) injected_failure: Option<String>,
    pub(crate) recovery_command: String,
    pub(crate) observability_assertions: Vec<ChaosQualificationObservabilityAssertion>,
    pub(crate) repro_command: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosQualificationObservabilityAssertion {
    pub(crate) code: String,
    pub(crate) passed: bool,
    pub(crate) signal: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ChaosScenarioSummary {
    pub(crate) name: String,
    pub(crate) status: ChaosScenarioStatus,
    pub(crate) failure_point: String,
    pub(crate) invariant: String,
    pub(crate) boundary_mode: String,
    pub(crate) expected_safety_property: String,
    pub(crate) proof_command: String,
    pub(crate) recovery_command: Option<String>,
    pub(crate) evidence: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ChaosScenarioStatus {
    CoveredByTests,
}
