use super::*;

#[test]
fn quarantine_replay_contract_distinguishes_exact_seek_availability() {
    let located = quarantine_replay_safety_contract_with_boundary(false, true, None, 1);
    let not_located = quarantine_replay_safety_contract_with_boundary(false, true, None, 0);

    assert!(located.redelivery_gate.contains("seek the durable stream"));
    assert!(located.next_operator_action.contains("exact seek command"));
    assert!(not_located
        .redelivery_gate
        .contains("locate the durable stream"));
    assert!(not_located.next_operator_action.contains("locate command"));
    assert!(not_located.cleared_state.contains("no applied transaction"));
    assert!(located.boundary_evidence.exact_seek_available);
    assert_eq!(located.boundary_evidence.exact_seek_command_count, 1);
    assert_eq!(not_located.boundary_evidence.boundary_status, "not_located");
}

#[test]
fn quarantine_replay_contract_preserves_boundary_evidence() {
    let boundary = LocalStreamLocateBoundarySummary {
        mode: "partitioned_scale_mode".to_string(),
        complete: false,
        status: "incomplete_boundary".to_string(),
        required_message_kinds: vec![
            "manifest".to_string(),
            "commit_marker".to_string(),
            "partition_chunk".to_string(),
        ],
        found_message_kinds: vec!["manifest".to_string(), "commit_marker".to_string()],
        missing_message_kinds: vec!["partition_chunk".to_string()],
        metadata_conflicts: vec!["partition_count differs across markers".to_string()],
        participating_partition_count: Some(4),
        found_partition_ids: Vec::new(),
        missing_partition_ids: vec![0, 1, 2, 3],
        found_partition_count: 0,
        missing_partition_count: Some(4),
    };

    let contract = quarantine_replay_safety_contract_with_boundary(true, true, Some(&boundary), 0);

    assert_eq!(
        contract.boundary_evidence.boundary_status,
        "incomplete_boundary"
    );
    assert_eq!(
        contract.boundary_evidence.boundary_mode.as_deref(),
        Some("partitioned_scale_mode")
    );
    assert_eq!(contract.boundary_evidence.complete, Some(false));
    assert_eq!(
        contract.boundary_evidence.required_message_kinds,
        vec!["manifest", "commit_marker", "partition_chunk"]
    );
    assert_eq!(
        contract.boundary_evidence.missing_message_kinds,
        vec!["partition_chunk"]
    );
    assert_eq!(
        contract.boundary_evidence.found_partition_ids,
        Vec::<u32>::new()
    );
    assert_eq!(
        contract.boundary_evidence.missing_partition_ids,
        vec![0, 1, 2, 3]
    );
    assert_eq!(
        contract.boundary_evidence.metadata_conflicts,
        vec!["partition_count differs across markers"]
    );
    assert!(!contract.boundary_evidence.exact_seek_available);
    assert!(contract
        .redelivery_gate
        .contains("locate the durable stream transaction boundary"));
}

#[test]
fn quarantine_replay_ready_refuses_unknown_transaction_boundary() {
    let transaction = TransactionKey {
        source_id: "source-a".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        transaction_id: "tx-missing".to_string(),
        commit_lsn: "0/16B6D28".to_string(),
    };

    let error =
        require_quarantine_replay_record(&transaction, None).expect_err("missing quarantine");
    match error {
        CliError::InvalidConfig(message) => {
            assert!(message.contains("requires an existing quarantined transaction"));
            assert!(message.contains("transaction_id=tx-missing"));
            assert!(message.contains("commit_lsn=0/16B6D28"));
            assert!(message.contains("exact transaction boundary"));
        }
        other => panic!("unexpected error: {other}"),
    }
}
