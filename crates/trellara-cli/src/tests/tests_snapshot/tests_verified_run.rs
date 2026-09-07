use super::*;

#[test]
fn verified_snapshot_run_promotes_handoff_ready_snapshot() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let run = SnapshotRun {
        failure_reason: Some("stale failure".to_string()),
        ..handoff_ready_run(&config, "snapshot-run-1")
    };
    let handoff = snapshot_handoff_event(&config, "0/16B8000");

    let verified = verified_snapshot_run(&config, &run, Some(&handoff)).expect("verified snapshot");

    assert_eq!(verified.state, SnapshotRunState::Verified);
    assert_eq!(verified.run_id, "snapshot-run-1");
    assert_eq!(verified.slot_name, "trellara_snapshot_slot");
    assert_eq!(verified.consistent_lsn.as_deref(), Some("0/16B8000"));
    assert_eq!(verified.copied_rows, 40);
    assert_eq!(verified.failure_reason, None);
}

#[test]
fn verified_snapshot_run_rejects_incomplete_or_other_flow_snapshot() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let run = SnapshotRun {
        state: SnapshotRunState::CopyingTable,
        current_relation: Some("public.sales".to_string()),
        copied_rows: 10,
        ..handoff_ready_run(&config, "snapshot-run-1")
    };

    let handoff = snapshot_handoff_event(&config, "0/16B8000");
    assert!(verified_snapshot_run(&config, &run, Some(&handoff)).is_none());

    let other_flow = SnapshotRun {
        state: SnapshotRunState::StreamHandoffReady,
        source_id: "other-source".to_string(),
        current_relation: None,
        ..run
    };
    assert!(verified_snapshot_run(&config, &other_flow, Some(&handoff)).is_none());
}

#[test]
fn verified_snapshot_run_requires_matching_handoff_evidence() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let run = handoff_ready_run(&config, "snapshot-run-1");

    assert!(verified_snapshot_run(&config, &run, None).is_none());

    let mismatched_lsn = snapshot_handoff_event(&config, "0/16B9000");
    assert!(verified_snapshot_run(&config, &run, Some(&mismatched_lsn)).is_none());

    let wrong_flow_handoff = SnapshotHandoffEvent {
        source_id: "other-source".to_string(),
        ..snapshot_handoff_event(&config, "0/16B8000")
    };
    assert!(verified_snapshot_run(&config, &run, Some(&wrong_flow_handoff)).is_none());

    let missing_consistent_lsn = SnapshotRun {
        consistent_lsn: None,
        ..run
    };
    let handoff = snapshot_handoff_event(&config, "0/16B8000");
    assert!(verified_snapshot_run(&config, &missing_consistent_lsn, Some(&handoff)).is_none());
}
