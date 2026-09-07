use crate::{
    ChaosFleetFanInSimulationSummary, ChaosPerformanceEnvelope,
    ChaosQualificationObservabilityAssertion, ChaosQualificationSimulationSummary,
    ChaosReportMetadata, ChaosSimulationSummary, ChaosSnapshotSimulationSummary,
    ChaosStrictChunkSimulationSummary, CORRECTNESS_REPORT_ARTIFACT, CORRECTNESS_REPORT_VERSION,
    QUICKSTART_ESTIMATED_MINUTES, QUICKSTART_TIME_BUDGET_MINUTES,
};

impl Default for ChaosReportMetadata {
    fn default() -> Self {
        Self {
            artifact: CORRECTNESS_REPORT_ARTIFACT.to_string(),
            report_version: CORRECTNESS_REPORT_VERSION.to_string(),
            package_version: env!("CARGO_PKG_VERSION").to_string(),
            source_revision: "local".to_string(),
            source_repository: "local".to_string(),
            workflow_run_url: "local".to_string(),
            generated_by: "trellara chaos report".to_string(),
            freshness_check: "make verify-correctness-report".to_string(),
        }
    }
}

impl Default for ChaosPerformanceEnvelope {
    fn default() -> Self {
        Self {
            quickstart_estimated_minutes: QUICKSTART_ESTIMATED_MINUTES,
            quickstart_time_budget_minutes: QUICKSTART_TIME_BUDGET_MINUTES,
            default_relay_max_transactions: 100,
            default_apply_max_messages: 100,
            stream_spill_threshold_changes:
                trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
            stream_spill_location:
                "quickstart configs set source.stream_spill_dir to ./target/trellara-spill"
                    .to_string(),
            bounded_large_transaction_mode:
                "pgoutput protocol v2 streaming with Trellara manifest and commit marker barriers"
                    .to_string(),
            local_transport_ack: "source acknowledgement advances only after local durable append"
                .to_string(),
            local_transport_replay:
                "stream inspection rebuilds missing, stale, or corrupt sidecar indexes from durable log frames and exposes last valid offsets plus seek guidance for replay"
                    .to_string(),
        }
    }
}

impl ChaosSimulationSummary {
    pub(crate) fn from_report(report: trellara_sim::SimulationReport) -> Self {
        Self {
            seed: report.seed,
            failure_point: report.failure_point,
            passed: report.passed,
            transaction_count: report.transaction_count,
            applied_transactions: report.applied_transactions,
            skipped_duplicates: report.skipped_duplicates,
            source_acknowledged_lsn: report.source_acknowledged_lsn,
            relay_durable_lsn: report.relay_durable_lsn,
            target_applied_lsn: report.target_applied_lsn,
            injected_failure: report.injected_failure,
            repro_command: format!(
                "cargo test -p trellara-sim {}",
                report.failure_point.as_str()
            ),
        }
    }
}

impl ChaosSnapshotSimulationSummary {
    pub(crate) fn from_report(report: trellara_sim::SnapshotSimulationReport) -> Self {
        Self {
            seed: report.seed,
            failure_point: report.failure_point,
            passed: report.passed,
            table_count: report.table_count,
            copied_tables: report.copied_tables,
            writes_after_snapshot: report.writes_after_snapshot,
            stream_replayed_transactions: report.stream_replayed_transactions,
            verification_matched: report.verification_matched,
            contract_refreshed: report.contract_refreshed,
            injected_failure: report.injected_failure,
            repro_command: format!(
                "cargo test -p trellara-sim {}",
                report.failure_point.as_str()
            ),
        }
    }
}

impl ChaosStrictChunkSimulationSummary {
    pub(crate) fn from_report(report: trellara_sim::StrictChunkSimulationReport) -> Self {
        Self {
            seed: report.seed,
            failure_point: report.failure_point,
            passed: report.passed,
            chunk_count: report.chunk_count,
            chunks_published: report.chunks_published,
            duplicate_chunks: report.duplicate_chunks,
            manifest_published: report.manifest_published,
            applied_transactions: report.applied_transactions,
            source_acknowledged_lsn: report.source_acknowledged_lsn,
            target_applied_lsn: report.target_applied_lsn,
            injected_failure: report.injected_failure,
            repro_command: format!(
                "cargo test -p trellara-sim {}",
                report.failure_point.as_str()
            ),
        }
    }
}

impl ChaosFleetFanInSimulationSummary {
    pub(crate) fn from_report(report: trellara_sim::FleetFanInSimulationReport) -> Self {
        Self {
            seed: report.seed,
            failure_point: report.failure_point,
            passed: report.passed,
            epoch_id: report.epoch_id,
            required_source_count: report.required_source_count,
            complete_source_count: report.complete_source_count,
            missing_source_count: report.missing_source_count,
            quarantined_source_count: report.quarantined_source_count,
            transaction_count: report.transaction_count,
            change_count: report.change_count,
            duplicate_replay_count: report.duplicate_replay_count,
            initial_state: report.initial_state,
            recovered_state: report.recovered_state,
            verification_status: report.verification_status,
            injected_failure: report.injected_failure,
            repro_command: format!(
                "cargo test -p trellara-sim {}",
                report.failure_point.as_str()
            ),
        }
    }
}

impl ChaosQualificationSimulationSummary {
    pub(crate) fn from_report(report: trellara_sim::QualificationSimulationReport) -> Self {
        Self {
            seed: report.seed,
            failure_point: report.failure_point,
            durable_boundary: report.durable_boundary,
            invariant: report.invariant,
            passed: report.passed,
            transaction_count: report.transaction_count,
            applied_transactions: report.applied_transactions,
            duplicate_replays: report.duplicate_replays,
            source_acknowledged_lsn: report.source_acknowledged_lsn,
            durable_lsn: report.durable_lsn,
            target_applied_lsn: report.target_applied_lsn,
            soak_hours: report.soak_hours,
            large_transaction_change_count: report.large_transaction_change_count,
            peak_memory_mib: report.peak_memory_mib,
            memory_ceiling_mib: report.memory_ceiling_mib,
            injected_failure: report.injected_failure,
            recovery_command: report.recovery_command,
            observability_assertions: report
                .observability_assertions
                .into_iter()
                .map(ChaosQualificationObservabilityAssertion::from)
                .collect(),
            repro_command: format!(
                "cargo test -p trellara-sim {}",
                report.failure_point.as_str()
            ),
        }
    }
}

impl From<trellara_sim::QualificationObservabilityAssertion>
    for ChaosQualificationObservabilityAssertion
{
    fn from(assertion: trellara_sim::QualificationObservabilityAssertion) -> Self {
        Self {
            code: assertion.code,
            passed: assertion.passed,
            signal: assertion.signal,
        }
    }
}
