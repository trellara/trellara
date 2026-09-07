use super::*;

pub(super) fn assert_local_flow(summary: &FleetReportSummary, local_path: &Path) {
    let local_flow = summary
        .flows
        .iter()
        .find(|flow| flow.config == local_path.display().to_string())
        .expect("local flow");

    assert_eq!(local_flow.mode, "strict_chunked_transaction_order");
    assert_eq!(local_flow.stream_kind, "local");
    assert!(local_flow
        .topics
        .contains(&"trellara.local-source.retail-sales.strict".to_string()));
    assert!(local_flow
        .topics
        .contains(&"trellara.local-source.retail-sales.manifest".to_string()));
    assert!(local_flow.transaction_boundary.contains("strict chunked"));
    assert_eq!(
        local_flow.lake_fanin.status,
        FleetLakeFaninStatus::PublishableWithGaps
    );
    assert_eq!(
        local_flow.lake_fanin.fanin_mode,
        "strict_chunked_epoch_fanin"
    );
    assert!(local_flow
        .lake_fanin
        .guidance
        .iter()
        .any(|guidance| guidance.contains("missing source schema fingerprints")));
    assert_local_convergence_gates(local_flow);
    assert_local_recovery_drills(local_flow);
}

fn assert_local_convergence_gates(local_flow: &FleetFlowSummary) {
    assert!(local_flow.convergence_gates.iter().any(|gate| {
        gate.code == "local_stream_durability"
            && gate.status == FleetConvergenceGateStatus::NeedsLiveEvidence
            && gate.proof_command.contains("stream inspect-local")
    }));
    assert!(local_flow.convergence_gates.iter().any(|gate| {
        gate.code == "target_convergence" && gate.proof_command.contains("trellara verify")
    }));
}

fn assert_local_recovery_drills(local_flow: &FleetFlowSummary) {
    assert!(local_flow.recovery_drills.iter().any(|drill| {
        drill.code == "target_quarantine_replay"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("quarantine replay-ready"))
            && drill
                .commands
                .iter()
                .any(|command| command.contains("stream seek-local"))
    }));
    assert!(local_flow.recovery_drills.iter().any(|drill| {
        drill.code == "checksum_reseed"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("reseed"))
    }));
    assert!(local_flow.recovery_drills.iter().any(|drill| {
        drill.code == "lake_offline_source_gap_acceptance"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("offline-stores-publish-with-gaps"))
            && drill.success_evidence.contains("source_watermarks")
    }));
    assert!(local_flow.recovery_drills.iter().any(|drill| {
        drill.code == "lake_late_source_recompute"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("late-store-recovery-completes-epoch"))
    }));
    assert!(local_flow.recovery_drills.iter().any(|drill| {
        drill.code == "lake_conflicting_duplicate_quarantine"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("conflicting-duplicate-quarantine"))
            && drill.operator_goal.contains("reseed")
    }));
}
