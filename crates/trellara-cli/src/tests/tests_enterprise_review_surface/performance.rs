use super::*;

#[test]
fn performance_envelope_names_strict_chunk_memory_and_durability_costs() {
    let config = TrellaraConfig::from_yaml(&local_strict_chunking_yaml(), "test").expect("config");
    config.validate().expect("valid strict chunk config");

    let summary = PerformanceEnvelopeSummary::from_config(&config, Path::new("local.yml"));

    assert_eq!(summary.mode, "strict_chunked_transaction_order");
    assert_eq!(summary.stream_kind, "local:fsync");
    assert_eq!(
        summary.quickstart_time_budget_minutes,
        QUICKSTART_TIME_BUDGET_MINUTES
    );
    assert_eq!(summary.stream_spill_threshold_changes, 1024);
    assert_eq!(
        summary.stream_spill_threshold_max_changes,
        trellara_pg_capture::MAX_STREAM_SPILL_THRESHOLD_CHANGES
    );
    assert!(summary
        .source_capture_contract
        .contains("pgoutput protocol_version=2 streaming=true"));
    assert!(summary
        .transaction_boundary_cost
        .contains("strict chunking limits relay memory"));
    assert!(summary
        .transport_durability_cost
        .contains("favors crash-safe source acknowledgement"));
    assert!(summary
        .expected_bottlenecks
        .iter()
        .any(|item| item.code == "streamed_transaction_spill"));
    assert!(summary
        .tuning_levers
        .iter()
        .any(|item| item.code == "stream_spill_threshold_changes"
            && item.evidence.contains("supported maximum 1000000")));
    assert!(summary
        .tuning_levers
        .iter()
        .any(|item| item.code == "strict_chunk_size"));
    assert!(summary
        .proof_commands
        .iter()
        .any(|command| command.contains("trellara stream inspect-local --config local.yml")));
}

#[test]
fn performance_envelope_explains_partitioned_scale_bottleneck() {
    let config = TrellaraConfig::from_yaml(&partitioned_yaml(), "test").expect("config");
    config.validate().expect("valid partitioned config");

    let summary = PerformanceEnvelopeSummary::from_config(&config, Path::new("partitioned.yml"));

    assert_eq!(summary.mode, "partitioned_scale_mode");
    assert!(summary
        .transaction_boundary_cost
        .contains("fans out across 16 partitions by store_id"));
    assert!(summary
        .expected_bottlenecks
        .iter()
        .any(|item| item.code == "partition_watermark_lag"));
    assert!(
        summary
            .tuning_levers
            .iter()
            .any(|item| item.code == "partition_count"
                && item.evidence.contains("partition_count=16"))
    );
    assert!(summary
        .proof_commands
        .contains(&"trellara partition-watermarks --config partitioned.yml".to_string()));
}
