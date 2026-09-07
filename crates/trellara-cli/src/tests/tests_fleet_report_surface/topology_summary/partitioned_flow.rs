use super::*;

pub(super) fn assert_partitioned_flow(summary: &FleetReportSummary, partitioned_path: &Path) {
    let partitioned_flow = summary
        .flows
        .iter()
        .find(|flow| flow.config == partitioned_path.display().to_string())
        .expect("partitioned flow");

    assert_eq!(partitioned_flow.mode, "partitioned_scale_mode");
    assert_eq!(
        partitioned_flow.lake_fanin.status,
        FleetLakeFaninStatus::PublishableWithGaps
    );
    assert_eq!(
        partitioned_flow.lake_fanin.fanin_mode,
        "partitioned_scale_epoch_fanin"
    );
    assert!(partitioned_flow
        .lake_fanin
        .source_watermark_rollup
        .contains("global low watermark"));
    assert!(partitioned_flow
        .lake_fanin
        .iceberg_checkpoint_receipt_gate
        .contains("matching checkpoint receipt"));
    assert!(partitioned_flow
        .lake_fanin
        .iceberg_checkpoint_receipt_gate
        .contains("partial table receipts keep the epoch invisible"));
    assert!(partitioned_flow
        .lake_fanin
        .straggler_policy
        .contains("publish_with_gaps"));
    assert_lake_fanin_commands(partitioned_flow);
    assert_partitioned_recovery(partitioned_flow);
    assert!(partitioned_flow
        .risks
        .iter()
        .any(|risk| risk.contains("partitioned scale requires manifest")));
}

pub(super) fn assert_fleet_lake_proof_commands(summary: &FleetReportSummary) {
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("lake fanin verify")));
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("partition_manifest_transaction_mismatch_fails_closed")));
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("lake spark-template dashboard")));
}

fn assert_lake_fanin_commands(partitioned_flow: &FleetFlowSummary) {
    assert!(partitioned_flow
        .lake_fanin
        .proof_commands
        .iter()
        .any(|command| command.contains("lake fanin verify")));
    assert!(partitioned_flow
        .lake_fanin
        .proof_commands
        .iter()
        .any(|command| command.contains("partition_manifest_duplicate_partition_id_fails_closed")));
    assert!(partitioned_flow
        .lake_fanin
        .proof_commands
        .iter()
        .any(|command| command.contains("manifest_boundary_mismatch_blocks_epoch_completion")));
    assert!(partitioned_flow
        .lake_fanin
        .proof_commands
        .iter()
        .any(|command| command.contains("lake spark-template dashboard")));
    assert!(partitioned_flow
        .proof_commands
        .iter()
        .any(|command| command.contains("partition-watermarks")));
    assert!(partitioned_flow
        .proof_commands
        .iter()
        .any(|command| command.contains("lake fanin verify")));
    assert!(partitioned_flow
        .proof_commands
        .iter()
        .any(|command| command.contains("partition_manifest_transaction_mismatch_fails_closed")));
    assert!(partitioned_flow
        .proof_commands
        .iter()
        .any(|command| command.contains("lake spark-template dashboard")));
}

fn assert_partitioned_recovery(partitioned_flow: &FleetFlowSummary) {
    assert!(partitioned_flow.convergence_gates.iter().any(|gate| {
        gate.code == "partition_watermarks"
            && gate.evidence.contains("global current-state view")
            && gate.proof_command.contains("partition-watermarks")
    }));
    assert!(partitioned_flow.recovery_drills.iter().any(|drill| {
        drill.code == "partition_barrier_replay"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("partition-watermarks"))
            && drill
                .commands
                .iter()
                .any(|command| command.contains("redeliver or seek Kafka topics"))
    }));
    assert!(partitioned_flow.recovery_drills.iter().any(|drill| {
        drill.code == "lake_missing_manifest_chunk_replay"
            && drill
                .commands
                .iter()
                .any(|command| command.contains("partition-watermarks"))
            && drill.success_evidence.contains("global low watermark")
    }));
}
