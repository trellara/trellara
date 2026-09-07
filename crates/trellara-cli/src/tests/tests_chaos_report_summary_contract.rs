use super::*;

pub(super) fn assert_chaos_report_summary_contract(summary: &ChaosRunSummary) {
    assert!(summary.passed);
    assert_eq!(summary.scenario_count, 63);
    assert_eq!(summary.covered_scenarios, 63);
    assert_eq!(summary.invariant_count, 60);
    assert_eq!(summary.deterministic_seed, 20_260_812);
    assert_eq!(summary.simulation_count, 28);
    assert!(summary.simulations_passed);
    assert_eq!(summary.metadata.artifact, CORRECTNESS_REPORT_ARTIFACT);
    assert_eq!(summary.metadata.report_version, CORRECTNESS_REPORT_VERSION);
    assert_eq!(summary.metadata.source_revision, "local");
    assert_eq!(summary.metadata.generated_by, "trellara chaos report");
    assert_eq!(
        summary.performance_envelope.quickstart_time_budget_minutes,
        QUICKSTART_TIME_BUDGET_MINUTES
    );
    assert_eq!(
        summary.performance_envelope.stream_spill_threshold_changes,
        trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES
    );
    assert!(summary
        .performance_envelope
        .stream_spill_location
        .contains("./target/trellara-spill"));
    assert!(summary
        .performance_envelope
        .bounded_large_transaction_mode
        .contains("manifest and commit marker barriers"));
    assert!(summary
        .performance_envelope
        .local_transport_replay
        .contains("last valid offsets"));
    assert_eq!(summary.enterprise_review_gates.len(), 8);
    assert!(summary.enterprise_review_gates.iter().any(|gate| {
        gate.code == "transaction_boundary"
            && gate
                .proof_surface
                .contains("inspect-transaction --file <envelope.pb> --format text")
            && gate.gate.contains("manifest counts bind")
    }));
    assert!(summary.enterprise_review_gates.iter().any(|gate| {
        gate.code == "recoverability"
            && gate
                .proof_surface
                .contains("status --config <flow> --view diagnostics --format text")
    }));
    assert!(summary.enterprise_review_gates.iter().any(|gate| {
        gate.code == "lane_d_qualification"
            && gate
                .proof_surface
                .contains("chaos report --output docs/correctness-report.html")
            && gate.gate.contains("24-hour large-transaction soak")
    }));
    assert!(summary
        .simulations
        .iter()
        .all(|simulation| simulation.passed
            && simulation.applied_transactions == simulation.transaction_count));
    assert!(summary
        .snapshot_simulations
        .iter()
        .all(|simulation| simulation.passed
            && simulation.verification_matched
            && simulation.copied_tables == simulation.table_count
            && simulation.stream_replayed_transactions == simulation.writes_after_snapshot));
    assert!(summary.snapshot_simulations.iter().any(|simulation| {
        simulation.failure_point == SnapshotFailurePoint::HandoffRecordedBeforeStreamStart
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert!(summary.snapshot_simulations.iter().any(|simulation| {
        simulation.failure_point == SnapshotFailurePoint::DdlDuringTableCopy
            && simulation.contract_refreshed
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert!(summary.strict_chunk_simulations.iter().all(|simulation| {
        simulation.passed
            && simulation.manifest_published
            && simulation.chunks_published == simulation.chunk_count
            && simulation.applied_transactions == 1
    }));
    assert!(summary.strict_chunk_simulations.iter().any(|simulation| {
        simulation.failure_point == StrictChunkFailurePoint::RelayCrashAfterChunksBeforeManifest
            && simulation.duplicate_chunks == simulation.chunk_count
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert!(summary.strict_chunk_simulations.iter().any(|simulation| {
        simulation.failure_point == StrictChunkFailurePoint::TargetCrashAfterStagingBeforeCommit
            && simulation.applied_transactions == 1
            && simulation.target_applied_lsn == simulation.source_acknowledged_lsn
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert_eq!(summary.fleet_fanin_simulations.len(), 4);
    assert!(summary
        .fleet_fanin_simulations
        .iter()
        .all(|simulation| { simulation.passed && simulation.required_source_count == 12 }));
    assert!(summary.fleet_fanin_simulations.iter().any(|simulation| {
        simulation.failure_point == FleetFanInFailurePoint::OfflineStoresPublishWithGaps
            && simulation.initial_state == trellara_lake::LakeCompletenessState::CompleteWithGaps
            && simulation.missing_source_count == 3
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert!(summary.fleet_fanin_simulations.iter().any(|simulation| {
        simulation.failure_point == FleetFanInFailurePoint::LateStoreRecoveryCompletesEpoch
            && simulation.initial_state == trellara_lake::LakeCompletenessState::CompleteWithGaps
            && simulation.recovered_state == Some(trellara_lake::LakeCompletenessState::Complete)
            && simulation.complete_source_count == 12
    }));
    assert!(summary.fleet_fanin_simulations.iter().any(|simulation| {
        simulation.failure_point == FleetFanInFailurePoint::DuplicateStoreTransactionReplay
            && simulation.duplicate_replay_count == 2
            && simulation.transaction_count == 9
    }));
    assert!(summary.fleet_fanin_simulations.iter().any(|simulation| {
        simulation.failure_point == FleetFanInFailurePoint::ConflictingDuplicateQuarantine
            && simulation.initial_state == trellara_lake::LakeCompletenessState::Quarantined
            && simulation.quarantined_source_count == 1
    }));
    assert_eq!(summary.qualification_simulations.len(), 5);
    assert!(summary.qualification_simulations.iter().all(|simulation| {
        simulation.passed
            && simulation.applied_transactions == simulation.transaction_count
            && simulation.source_acknowledged_lsn == simulation.durable_lsn
            && simulation.target_applied_lsn == simulation.durable_lsn
            && simulation
                .observability_assertions
                .iter()
                .all(|assertion| assertion.passed)
    }));
    assert!(summary.qualification_simulations.iter().any(|simulation| {
        simulation.failure_point == QualificationFailurePoint::SourcePromotionWhileRelayDisconnected
            && simulation.duplicate_replays == 1
            && simulation
                .observability_assertions
                .iter()
                .any(|assertion| assertion.code == "source_ack_lag_visible")
    }));
    assert!(summary.qualification_simulations.iter().any(|simulation| {
        simulation.failure_point == QualificationFailurePoint::BrokerOutageQuorumLoss
            && simulation.durable_boundary == "broker_quorum_publish_ack"
            && simulation
                .observability_assertions
                .iter()
                .any(|assertion| assertion.code == "broker_quorum_unavailable")
    }));
    assert!(summary.qualification_simulations.iter().any(|simulation| {
        simulation.failure_point == QualificationFailurePoint::TargetRestartDuringApply
            && simulation.duplicate_replays == 1
            && simulation
                .observability_assertions
                .iter()
                .any(|assertion| assertion.code == "target_checkpoint_not_advanced_early")
    }));
    assert!(summary.qualification_simulations.iter().any(|simulation| {
        simulation.failure_point == QualificationFailurePoint::ObjectStoreSuccessCatalogTimeout
            && simulation.durable_boundary == "object_store_write_before_catalog_visibility"
            && simulation
                .observability_assertions
                .iter()
                .any(|assertion| assertion.code == "catalog_commit_pending")
    }));
    assert!(summary.qualification_simulations.iter().any(|simulation| {
        simulation.failure_point
            == QualificationFailurePoint::TwentyFourHourSoakLargeTransactionMemoryCeiling
            && simulation.soak_hours == 24
            && simulation.peak_memory_mib <= simulation.memory_ceiling_mib
            && simulation
                .observability_assertions
                .iter()
                .any(|assertion| assertion.code == "large_transaction_memory_ceiling")
    }));
    assert!(summary.simulations.iter().any(|simulation| {
        simulation.failure_point == FailurePoint::StreamAckLossAfterApply
            && simulation.skipped_duplicates == 1
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert!(summary.simulations.iter().any(|simulation| {
        simulation.failure_point == FailurePoint::TargetQuarantineRepairReplay
            && simulation.repro_command.contains("trellara-sim")
    }));
    assert_eq!(summary.verification_command, "cargo test --workspace");
}
