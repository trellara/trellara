use super::*;

#[tokio::test]
async fn evaluate_command_renders_enterprise_review_path() {
    let root =
        std::env::temp_dir().join(format!("trellara-evaluate-command-{}", std::process::id()));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create evaluate temp dir");
    fs::write(&config_path, local_stream_yaml()).expect("write evaluate config");

    let output = execute(Cli {
        command: Command::Evaluate(EvaluateArgs {
            config: config_path,
            format: PilotGuideOutputFormat::Text,
        }),
    })
    .await
    .expect("evaluate output");

    assert!(output.contains("Trellara enterprise evaluation"));
    assert!(output.contains("recommended_mode: strict_transaction_order"));
    assert!(output.contains("verified replication posture"));
    assert!(output.contains("manifest boundary_mode"));
    assert!(output.contains("trellara run --local --config"));
    assert!(output.contains("trellara pilot-package --config"));
    assert!(output.contains("trellara stream inspect-local --config"));

    fs::remove_dir_all(root).expect("remove evaluate temp dir");
}

#[tokio::test]
async fn consistency_command_renders_flow_contract() {
    let root = std::env::temp_dir().join(format!(
        "trellara-consistency-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create consistency temp dir");
    fs::write(&config_path, local_strict_chunking_yaml()).expect("write consistency config");

    let output = execute(Cli {
        command: Command::Consistency(ConsistencyContractArgs {
            config: config_path,
            format: PilotGuideOutputFormat::Text,
        }),
    })
    .await
    .expect("consistency output");

    assert!(output.contains("Trellara consistency contract"));
    assert!(output.contains("source_capture_contract: pgoutput protocol_version=2"));
    assert!(output.contains("transaction_boundary_contract: strict chunked"));
    assert!(output.contains("source_ack_contract: source feedback advances only after"));
    assert!(output.contains("snapshot_handoff_contract: initial copy is tied"));
    assert!(output.contains("target_checkpoint_contract: target checkpoints advance only"));
    assert!(output.contains("strict_chunk_manifest"));
    assert!(output.contains("source_ack_after_durable_publish"));
    assert!(output.contains("trellara stream locate-local --config"));

    fs::remove_dir_all(root).expect("remove consistency temp dir");
}

#[tokio::test]
async fn performance_command_renders_configured_envelope() {
    let root = std::env::temp_dir().join(format!(
        "trellara-performance-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create performance temp dir");
    fs::write(&config_path, local_strict_chunking_yaml()).expect("write performance config");

    let output = execute(Cli {
        command: Command::Performance(PerformanceEnvelopeArgs {
            config: config_path,
            format: PilotGuideOutputFormat::Text,
        }),
    })
    .await
    .expect("performance output");

    assert!(output.contains("Trellara performance envelope"));
    assert!(output.contains("quickstart_target: 8 minute estimate within 10 minute budget"));
    assert!(output.contains("source_capture_contract: pgoutput protocol_version=2"));
    assert!(output.contains("stream_spill_threshold_changes: 1024"));
    assert!(output.contains("stream_spill_threshold_max_changes: 1000000"));
    assert!(output.contains("strict chunking limits relay memory"));
    assert!(output.contains("local fsync durability"));
    assert!(output.contains("streamed_transaction_spill"));
    assert!(output.contains("measurement_note: This envelope is configuration-derived"));

    fs::remove_dir_all(root).expect("remove performance temp dir");
}

#[tokio::test]
async fn identity_audit_command_renders_pk_and_toast_contract() {
    let root = std::env::temp_dir().join(format!(
        "trellara-identity-audit-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create identity audit temp dir");
    fs::write(&config_path, local_stream_yaml()).expect("write identity audit config");

    let output = execute(Cli {
        command: Command::IdentityAudit(IdentityAuditArgs {
            config: config_path,
            format: PilotGuideOutputFormat::Text,
        }),
    })
    .await
    .expect("identity audit output");

    assert!(output.contains("Trellara identity audit"));
    assert!(output.contains("ordinary_pk_tables_do_not_require_full: true"));
    assert!(output.contains("cdc_apply_gate: released: configured primary-key apply"));
    assert!(output.contains("[pk_apply_ready] public.sales"));
    assert!(output.contains("REPLICA IDENTITY FULL is not required"));
    assert!(output.contains("absent non-key columns are treated as unchanged"));
    assert!(output.contains("plans_update_with_key_predicate"));
    assert!(output.contains("update_omits_absent_non_key_columns_for_unchanged_toast"));
    assert!(output.contains("update_omits_explicit_unchanged_toast_marker"));

    fs::remove_dir_all(root).expect("remove identity audit temp dir");
}

#[tokio::test]
async fn semantics_command_renders_consumer_matrix() {
    let root = std::env::temp_dir().join(format!(
        "trellara-semantics-command-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("partitioned.yml");
    fs::create_dir_all(&root).expect("create semantics temp dir");
    fs::write(&config_path, partitioned_yaml()).expect("write semantics config");

    let output = execute(Cli {
        command: Command::Semantics(ConfigArgs {
            config: config_path,
        }),
    })
    .await
    .expect("semantics output");

    assert!(output.contains("\"dataset_mode\": \"partitioned_scale_mode\""));
    assert!(output.contains("\"selected_consumer_mode\": \"barrier_aware\""));
    assert!(output.contains("\"mode\": \"exact_transaction\""));
    assert!(output.contains("\"mode\": \"partition_local\""));
    assert!(output.contains("\"availability\": \"supported_with_barrier\""));
    assert!(output.contains("atomic visibility for cross-partition transactions"));
    assert!(output.contains("trellara partition-watermarks --config"));

    fs::remove_dir_all(root).expect("remove semantics temp dir");
}
