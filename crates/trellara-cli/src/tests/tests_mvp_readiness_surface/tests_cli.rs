use super::*;

#[test]
fn cli_parses_mvp_check_text_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "mvp-check",
        "--config",
        "examples/retail-fleet/local.yml",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::MvpCheck(MvpCheckArgs { config, format })
            if config.as_path() == Path::new("examples/retail-fleet/local.yml")
                && format == QuickstartOutputFormat::Text
    ));
}

#[tokio::test]
async fn mvp_check_command_renders_text_readiness() {
    let output = execute(Cli {
        command: Command::MvpCheck(MvpCheckArgs {
            config: workspace_path("examples/retail-fleet/local.yml"),
            format: QuickstartOutputFormat::Text,
        }),
    })
    .await
    .expect("mvp check output");

    assert_contains_all(
        &output,
        &[
            "Trellara MVP readiness",
            "ready: true",
            "criteria: 14/14 passed",
            "priority_next_commands:",
            "[pass] no_broker_verified_flow_under_10_minutes",
            "[pass] pgoutput_capture_path",
            "pgoutput.protocol_version=2",
            "streamed transaction scenario covered=true",
            "[pass] replica_identity_default_supported",
            "primary-key apply covered=true",
            "key-change apply covered=true",
            "unchanged TOAST preservation covered=true",
            "[pass] snapshot_stream_handoff_crash_safe",
            "10/10 snapshot proof scenarios covered",
            "[pass] large_transactions_bounded",
            "8/8 strict chunk proof scenarios covered",
            "5 strict chunk simulations pass",
            "applied_transaction_insert_fails_closed_on_duplicate_key",
            "[pass] partitioned_scale_mode_proven",
            "7/7 partitioned scale proof scenarios covered",
            "pending_stats_reports_invalid_commit_marker_separately",
            "[pass] source_failover_readiness_proven",
            "3/3 source failover proof scenarios covered",
            "[pass] schema_change_recovery_scripted",
            "3/3 schema-change proof scenarios covered",
            "[pass] ddl_propagation_contract_packaged",
            "schema-barrier propagation",
            "target/lake/Spark sink ACKs",
            "trellara schema ddl-apply-plan --config",
            "trellara schema ddl-envelope-plan --config",
            "trellara schema ddl-barrier status --config",
            "[pass] protocol_property_tests_cover_invariants",
            "modular envelope/idempotency/LSN/strict chunk/partitioned/missing/duplicate proptests current=true",
            "[pass] public_correctness_report_available",
            "cargo test --workspace before publishing",
            "[pass] design_partner_can_evaluate_without_kafka",
            "local-run-proof.md",
            "make quickstart-proof-check",
        ],
    );
    assert!(output.find("priority_next_commands:") < output.find("next_commands:"));
}
