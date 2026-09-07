use super::*;

#[test]
fn consistency_contract_names_strict_chunk_checkpoint_boundaries() {
    let config = TrellaraConfig::from_yaml(&local_strict_chunking_yaml(), "test").expect("config");
    config.validate().expect("valid strict chunk config");

    let summary =
        ConsistencyContractSummary::from_config(&config, Path::new("local.yml")).expect("summary");

    assert_eq!(summary.mode, "strict_chunked_transaction_order");
    assert_eq!(summary.selected_consumer_mode, "exact_transaction");
    assert_eq!(summary.stream_kind, "local:fsync");
    assert!(summary
        .topics
        .contains(&"trellara.local-source.retail-sales.strict".to_string()));
    assert!(summary
        .topics
        .contains(&"trellara.local-source.retail-sales.manifest".to_string()));
    assert!(summary
        .source_capture_contract
        .contains("pgoutput protocol_version=2 streaming=true"));
    assert!(summary
        .transaction_boundary_contract
        .contains("strict_chunk_manifest"));
    assert!(summary
        .source_ack_contract
        .contains("source feedback advances only after"));
    assert!(summary
        .snapshot_handoff_contract
        .contains("pgoutput slot consistent LSN"));
    assert!(summary
        .target_checkpoint_contract
        .contains("checkpoints advance only after"));
    assert!(summary
        .replay_contract
        .contains("strict chunks, strict_chunk_manifest, and commit marker"));
    assert!(summary.partition_contract.is_none());
    assert!(summary
        .invariants
        .iter()
        .any(|invariant| invariant.code == "local_replay_locates_exact_boundary"));
    assert!(summary
        .proof_commands
        .contains(&"trellara inspect-transaction --file <envelope.pb> --format text".to_string()));
}

#[test]
fn consistency_contract_explains_partitioned_visibility_tradeoff() {
    let config = TrellaraConfig::from_yaml(&local_partitioned_yaml(), "test").expect("config");
    config.validate().expect("valid partitioned config");

    let summary = ConsistencyContractSummary::from_config(&config, Path::new("partitioned.yml"))
        .expect("summary");

    assert_eq!(summary.mode, "partitioned_scale_mode");
    assert_eq!(summary.selected_consumer_mode, "barrier_aware");
    assert!(summary
        .transaction_boundary_contract
        .contains("manifest and commit marker preserve transaction identity"));
    assert!(summary
        .consumer_visibility_contract
        .contains("partition-local consumers must label lower-latency output as non-atomic"));
    assert!(summary
        .replay_contract
        .contains("every participating partition message"));
    let partition = summary
        .partition_contract
        .as_ref()
        .expect("partition contract");
    assert_eq!(partition.key_column, "store_id");
    assert_eq!(partition.partition_count, 16);
    assert_eq!(partition.null_key_policy, "quarantine");
    assert_eq!(partition.key_change_policy, "quarantine");
    assert_eq!(
        partition.manifest_topic,
        "trellara.local-source.retail-sales.manifest"
    );
    assert!(partition
        .global_visibility_rule
        .contains("every participating partition watermark"));
    assert!(partition
        .partition_local_visibility_rule
        .contains("must not claim atomic visibility"));
    assert!(summary
        .invariants
        .iter()
        .any(|invariant| invariant.code == "partition_watermarks_gate_global_visibility"));
    assert!(summary
        .invariants
        .iter()
        .any(|invariant| invariant.code == "local_partition_barrier_reconstructs_before_ack"));
    assert!(summary.proof_commands.contains(
        &"trellara stream reconstruct-local --config partitioned.yml --transaction-id <tx> --commit-lsn <lsn>"
            .to_string()
    ));
    assert!(summary
        .proof_commands
        .contains(&"trellara partition-watermarks --config partitioned.yml".to_string()));
}
