use super::*;

#[test]
fn run_summary_text_renders_human_proof_chain() {
    let config =
        TrellaraConfig::from_yaml(&local_strict_chunking_yaml(), "test").expect("parse config");
    let snapshot = fixture_snapshot_copy_summary();
    let bootstrap = verified_bootstrap_summary();
    let relay = relay_summary(1);
    let apply = barrier_pending_apply_summary();
    let verify = matching_verify_summary();
    let proof_chain = run_proof_chain(
        Path::new("trellara.yml"),
        &config,
        Some(&snapshot),
        &bootstrap,
        &relay,
        &apply,
        Some(&verify),
    );
    let summary = RunSummary {
        config: "trellara.yml".to_string(),
        source_id: "local-source".to_string(),
        dataset_id: "retail-sales".to_string(),
        mode: "strict_transaction_order".to_string(),
        stream_kind: "local".to_string(),
        bootstrap,
        apply_schema_ready: true,
        snapshot: Some(snapshot),
        relay,
        apply,
        verify: Some(verify),
        proof_chain,
        next_commands: vec!["trellara status --config trellara.yml".to_string()],
    };

    let output = render_run_summary_text(&summary);

    assert!(output.contains("Trellara local run"));
    assert!(output.contains("phase_summary:"));
    assert!(output.contains("- apply: 1 transactions, 2 changes, 3 acked_messages"));
    assert!(output.contains("- verify: converged=true checksum_status=match"));
    assert!(output.contains("target_relation=public.sales relation_match=true"));
    assert!(output.contains(
        "barrier_pending: transactions=1 missing_manifest=0 missing_commit_marker=1 invalid_commit_marker=0 missing_chunks=0 extra_chunks=0"
    ));
    assert!(output.contains("barrier_pending_blockers: 1 transaction(s) missing commit marker"));
    assert!(output.contains("barrier_pending_blocker_codes: missing_commit_marker"));
    assert!(output.contains(
        "barrier_pending_recovery_actions: replay commit-marker topic for pending barrier transactions before target apply"
    ));
    assert!(output.contains("proof_chain:"));
    assert!(output.contains("[verified] snapshot_handoff_boundary"));
    assert!(output.contains("[verified] bounded_large_transaction_capture"));
    assert!(output.contains("[verified] source_ack_after_local_durability"));
    assert!(output.contains("trellara stream inspect-local --config trellara.yml"));
    assert!(output.contains(
        "invalid_commit_marker=0, missing_chunks=0, extra_chunks=0, barrier_pending_blockers=1 transaction(s) missing commit marker, barrier_pending_blocker_codes=missing_commit_marker, barrier_pending_recovery_actions=replay commit-marker topic for pending barrier transactions before target apply"
    ));
    assert!(output.contains("next_commands:"));
}
