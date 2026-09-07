use super::*;

#[test]
fn run_proof_chain_marks_verified_local_run_gates() {
    let config =
        TrellaraConfig::from_yaml(&local_strict_chunking_yaml(), "test").expect("parse config");
    let snapshot = fixture_snapshot_copy_summary();
    let bootstrap = verified_bootstrap_summary();
    let relay = relay_summary(2);
    let apply = apply_summary(2);
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

    assert_eq!(proof_chain.len(), 6);
    assert!(proof_chain
        .iter()
        .all(|gate| gate.status == RunProofStatus::Verified));
    assert!(gate(&proof_chain, "snapshot_handoff_boundary")
        .evidence
        .contains("stream_handoff_ready"));
    assert!(gate(&proof_chain, "snapshot_handoff_boundary")
        .evidence
        .contains("skipped_tables=0"));
    assert!(gate(&proof_chain, "snapshot_handoff_boundary")
        .evidence
        .contains("public.sales:copy_complete rows=12 watermark_lsn=0/16B8000"));
    assert!(gate(&proof_chain, "snapshot_handoff_boundary")
        .evidence
        .contains("next=trellara relay --config <config>"));
    assert!(gate(&proof_chain, "snapshot_handoff_boundary")
        .evidence
        .contains("note=snapshot reached stream handoff"));
    let source_ack = &gate(&proof_chain, "source_ack_after_local_durability").evidence;
    assert!(source_ack.contains("2 durable local messages"));
    assert!(source_ack.contains("source_ack_lsn=0/16B9000"));
    assert!(source_ack.contains("publish_destinations_match=true"));
    assert!(source_ack.contains("publish_destination_count=1"));
    assert!(source_ack.contains("every Trellara publish ack is durable"));
    assert!(source_ack.contains("local_stream_status=clean"));
    assert!(source_ack.contains("total_messages=2 pending_messages=0"));
    assert!(source_ack.contains(
        "last_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack"
    ));
    assert!(source_ack.contains("last_publish_ack_topic=trellara.local-source.sales.strict"));
    assert!(source_ack.contains("last_publish_ack_offset=1"));
    assert!(source_ack.contains("last_publish_ack_durability=fsync"));
    assert!(source_ack.contains("last_publish_ack_crash_safe_ack=true"));
    assert!(source_ack.contains("last_publish_ack_indexed=true"));
    assert!(source_ack.contains("last_publish_ack_replayable=true"));
    assert!(source_ack.contains("last_publish_ack_index_status=healthy"));
    assert!(source_ack.contains("last_publish_ack_torn_tail_bytes=0"));
    assert!(source_ack.contains(
        "source_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack"
    ));
    assert!(source_ack.contains("source_ack_durability=fsync"));
    assert!(source_ack.contains("source_ack_crash_safe_ack=true"));
    assert!(source_ack.contains("source_ack_expected_publish_messages=2"));
    assert!(source_ack.contains("source_ack_durable_publish_acks=2"));
    assert!(source_ack.contains("source_ack_all_publish_acks_proven=true"));
    assert!(source_ack.contains("source_ack_proofed_ack_count=2"));
    assert!(source_ack.contains(
        "source_ack_proofed_destinations=trellara.local-source.sales.strict:0:0,trellara.local-source.sales.strict:0:1"
    ));
    assert!(gate(&proof_chain, "bounded_large_transaction_capture")
        .evidence
        .contains("pgoutput.streaming=true"));
    assert!(gate(&proof_chain, "bounded_large_transaction_capture")
        .evidence
        .contains("strict_chunking.max_changes_per_chunk=1000"));
    assert!(gate(&proof_chain, "bounded_large_transaction_capture")
        .evidence
        .contains("bounded_memory_contract=spill_then_strict_chunk_manifest"));
    assert!(gate(&proof_chain, "bounded_large_transaction_capture")
        .evidence
        .contains("visibility_contract=commit_marker_waits_for_complete_strict_chunk_set"));
    assert!(gate(&proof_chain, "bounded_large_transaction_capture")
        .evidence
        .contains("parallel_replay_contract=parallel replay is disabled"));
    assert!(gate(&proof_chain, "convergence_verification")
        .evidence
        .contains("converged=true"));
    assert!(gate(&proof_chain, "convergence_verification")
        .evidence
        .contains("target_relation=public.sales"));
    assert!(gate(&proof_chain, "convergence_verification")
        .evidence
        .contains("relation_match=true"));
    assert!(gate(&proof_chain, "convergence_verification")
        .evidence
        .contains("evidence_sha256="));
}

#[tokio::test]
async fn local_run_stream_evidence_collects_source_ack_and_last_ack_proofs() {
    let root = std::env::temp_dir().join(format!(
        "trellara-cli-local-run-proof-{}",
        unique_test_suffix()
    ));
    let yaml = local_stream_yaml().replace(
        "path: /tmp/trellara-local-stream",
        &format!("path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse config");

    publish_local_strict_transactions(&config, 2).await;
    let topic = config
        .local_stream_topics()
        .expect("local stream topics")
        .remove(0);
    let latest_publish_messages = vec![
        trellara_stream_local::read_local_message_at(&root, &topic, 0)
            .expect("read first")
            .expect("first message"),
        trellara_stream_local::read_local_message_at(&root, &topic, 1)
            .expect("read second")
            .expect("second message"),
    ];
    let latest_publish_acks = vec![
        trellara_stream::PublishAck {
            topic: topic.clone(),
            partition: 0,
            offset: 0,
        },
        trellara_stream::PublishAck {
            topic: topic.clone(),
            partition: 0,
            offset: 1,
        },
    ];
    let evidence = collect_local_run_stream_evidence(
        &config,
        &latest_publish_messages,
        &latest_publish_acks,
        Some(&topic),
        Some(0),
        Some(1),
    )
    .expect("collect local run stream evidence");
    let source_ack_proof = evidence
        .source_ack_durability_proof
        .expect("source ACK durability proof");
    let proof = evidence
        .last_publish_ack_proof
        .expect("last publish ACK proof");

    assert_eq!(evidence.status, "clean");
    assert_eq!(
        source_ack_proof.contract,
        trellara_stream_local::LOCAL_SOURCE_ACK_DURABILITY_CONTRACT
    );
    assert_eq!(source_ack_proof.durability, "fsync");
    assert!(source_ack_proof.crash_safe_ack);
    assert_eq!(source_ack_proof.expected_publish_messages, 2);
    assert_eq!(source_ack_proof.durable_publish_acks, 2);
    assert!(source_ack_proof.all_publish_acks_proven);
    assert_eq!(source_ack_proof.proofed_ack_count, 2);
    assert_eq!(source_ack_proof.publish_ack_proofs.len(), 2);
    assert_eq!(source_ack_proof.publish_ack_proofs[0].topic, topic);
    assert_eq!(source_ack_proof.publish_ack_proofs[0].partition, 0);
    assert_eq!(source_ack_proof.publish_ack_proofs[0].offset, 0);
    assert!(source_ack_proof
        .publish_ack_proofs
        .iter()
        .all(|proof| proof.indexed && proof.replayable && proof.torn_tail_bytes == 0));
    assert_eq!(
        proof.contract,
        trellara_stream_local::LOCAL_PUBLISH_ACK_PROOF_CONTRACT
    );
    assert_eq!(proof.topic, topic);
    assert_eq!(proof.partition, 0);
    assert_eq!(proof.offset, 1);
    assert_eq!(proof.durability, "fsync");
    assert!(proof.crash_safe_ack);
    assert!(proof.indexed);
    assert!(proof.replayable);
    assert_eq!(proof.index_status, "healthy");
    assert_eq!(proof.torn_tail_bytes, 0);

    fs::remove_dir_all(root).expect("remove local run proof temp dir");
}

#[test]
fn run_proof_chain_surfaces_pending_and_at_risk_evidence() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse config");
    let bootstrap = at_risk_bootstrap_summary();
    let relay = RelaySummary::default();
    let apply = ApplySummary::default();
    let verify = mismatched_verify_summary();

    let proof_chain = run_proof_chain(
        Path::new("trellara.yml"),
        &config,
        None,
        &bootstrap,
        &relay,
        &apply,
        Some(&verify),
    );

    let snapshot = gate(&proof_chain, "snapshot_handoff_boundary");
    assert_eq!(snapshot.status, RunProofStatus::NeedsEvidence);
    assert!(snapshot.evidence.contains("snapshot was skipped"));

    let bootstrap = gate(&proof_chain, "source_bootstrap_position");
    assert_eq!(bootstrap.status, RunProofStatus::AtRisk);
    assert!(bootstrap.evidence.contains("consistent_lsn=missing"));

    let bounded_large_transaction = gate(&proof_chain, "bounded_large_transaction_capture");
    assert_eq!(bounded_large_transaction.status, RunProofStatus::AtRisk);
    assert!(bounded_large_transaction
        .evidence
        .contains("missing_manifest_or_chunk_boundary"));
    assert!(bounded_large_transaction
        .evidence
        .contains("bounded_memory_contract=not_proven"));
    assert!(bounded_large_transaction
        .evidence
        .contains("visibility_contract=not_proven"));

    let convergence = gate(&proof_chain, "convergence_verification");
    assert_eq!(convergence.status, RunProofStatus::AtRisk);
    assert!(convergence.evidence.contains("checksum_status=Mismatch"));
}

#[test]
fn run_proof_chain_marks_snapshot_boundary_mismatch_at_risk() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse config");
    let mut snapshot = fixture_snapshot_copy_summary();
    snapshot.consistent_lsn = "0/16B7000".to_string();
    let bootstrap = verified_bootstrap_summary();

    let proof_chain = run_proof_chain(
        Path::new("trellara.yml"),
        &config,
        Some(&snapshot),
        &bootstrap,
        &RelaySummary::default(),
        &ApplySummary::default(),
        None,
    );

    let snapshot_gate = gate(&proof_chain, "snapshot_handoff_boundary");

    assert_eq!(snapshot_gate.status, RunProofStatus::AtRisk);
    assert!(snapshot_gate.evidence.contains("consistent_lsn=0/16B7000"));
    assert!(snapshot_gate
        .evidence
        .contains("bootstrap_consistent_lsn=0/16B8000"));
    assert!(snapshot_gate
        .evidence
        .contains("snapshot_handoff_blocker_codes=none"));
    assert!(snapshot_gate
        .evidence
        .contains("snapshot_handoff_recovery_actions=none"));
    assert!(snapshot_gate.evidence.contains(
        "handoff_proof_command=trellara snapshot --config <config> --run-id snapshot-run-1 --table public.sales"
    ));
}

#[test]
fn snapshot_boundary_gate_surfaces_handoff_recovery_fields() {
    let config = TrellaraConfig::from_yaml(&local_stream_yaml(), "test").expect("parse config");
    let mut snapshot = fixture_snapshot_copy_summary();
    snapshot.state = SnapshotRunState::CopyingTable.to_string();
    snapshot.handoff_blocker_codes = vec!["missing_table_progress".to_string()];
    snapshot.recovery_actions = vec![
        "rerun or resume the snapshot until every selected table records copy_complete".to_string(),
    ];
    let bootstrap = verified_bootstrap_summary();

    let proof_chain = run_proof_chain(
        Path::new("trellara.yml"),
        &config,
        Some(&snapshot),
        &bootstrap,
        &RelaySummary::default(),
        &ApplySummary::default(),
        None,
    );

    let snapshot_gate = gate(&proof_chain, "snapshot_handoff_boundary");

    assert_eq!(snapshot_gate.status, RunProofStatus::NeedsEvidence);
    assert!(snapshot_gate
        .evidence
        .contains("snapshot_handoff_blocker_codes=missing_table_progress"));
    assert!(snapshot_gate.evidence.contains(
        "snapshot_handoff_recovery_actions=rerun or resume the snapshot until every selected table records copy_complete"
    ));
}

#[test]
fn target_apply_checkpoint_gate_surfaces_pending_barrier_blockers() {
    let apply = barrier_pending_apply_summary();

    let gate = target_apply_checkpoint_run_proof(Path::new("trellara.yml"), &apply);

    assert_eq!(gate.status, RunProofStatus::Verified);
    assert!(gate.evidence.contains("pending_barrier_transactions=1"));
    assert!(gate
        .evidence
        .contains("barrier_pending_blockers=1 transaction(s) missing commit marker"));
    assert!(gate
        .evidence
        .contains("barrier_pending_blocker_codes=missing_commit_marker"));
    assert!(gate.evidence.contains(
        "barrier_pending_recovery_actions=replay commit-marker topic for pending barrier transactions before target apply"
    ));
}

#[test]
fn convergence_gate_requires_explicit_verify_evidence() {
    let pending = convergence_verification_run_proof(Path::new("trellara.yml"), None);
    assert_eq!(pending.status, RunProofStatus::NeedsEvidence);
    assert!(pending.evidence.contains("run did not include --verify"));

    let verified = convergence_verification_run_proof(
        Path::new("trellara.yml"),
        Some(&matching_verify_summary()),
    );
    assert_eq!(verified.status, RunProofStatus::Verified);
    assert!(verified.evidence.contains("checksum_status=Match"));
    assert!(verified.evidence.contains("target_caught_up=true"));
    assert!(verified.evidence.contains("relation=public.sales"));
    assert!(verified.evidence.contains("target_relation=public.sales"));
}

#[test]
fn convergence_gate_rejects_stale_target_watermark_even_with_matching_tables() {
    let mut verify = matching_verify_summary();
    verify.target_watermark_lsn = "0/16B8000".to_string();

    let gate = convergence_verification_run_proof(Path::new("trellara.yml"), Some(&verify));

    assert_eq!(gate.status, RunProofStatus::AtRisk);
    assert!(gate.evidence.contains("checksum_status=Match"));
    assert!(gate.evidence.contains("target_caught_up=false"));
    assert!(gate.evidence.contains("source_watermark_lsn=0/16B9000"));
    assert!(gate.evidence.contains("target_watermark_lsn=0/16B8000"));
}

#[test]
fn convergence_gate_requires_table_evidence_digest() {
    let mut verify = matching_verify_summary();
    verify.tables[0].evidence_sha256.clear();

    let gate = convergence_verification_run_proof(Path::new("trellara.yml"), Some(&verify));

    assert_eq!(gate.status, RunProofStatus::AtRisk);
    assert!(gate.evidence.contains("evidence_sha256="));
}

#[test]
fn convergence_gate_requires_table_relation_identity_evidence() {
    let mut aggregate_only = matching_verify_summary();
    aggregate_only.tables.clear();

    let gate = convergence_verification_run_proof(Path::new("trellara.yml"), Some(&aggregate_only));

    assert_eq!(gate.status, RunProofStatus::AtRisk);
    assert!(gate.evidence.contains("table_count=0"));
    assert!(gate.evidence.contains("table_evidence=none"));
}
