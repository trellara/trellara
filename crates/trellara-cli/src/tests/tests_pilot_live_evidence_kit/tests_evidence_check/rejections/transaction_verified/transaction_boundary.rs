use super::*;

#[test]
fn pilot_evidence_check_rejects_transaction_boundary_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-transaction-boundary-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50 last_durable_lsn=0/16B6C50\ntarget_checkpoint last_durable_lsn=0/16B6C50 last_applied_lsn=0/16B6C50\nsource_ack_lsn=0/16B6C50\nsource_ack_after_durable_publish=true\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=1\nevery Trellara publish ack is durable\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_durability=fsync source_ack_crash_safe_ack=true source_ack_expected_publish_messages=1 source_ack_durable_publish_acks=1 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=1\nlast_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack last_publish_ack_durability=fsync last_publish_ack_crash_safe_ack=true last_publish_ack_topic=trellara.local-source.retail-sales.strict last_publish_ack_partition=0 last_publish_ack_offset=0 last_publish_ack_indexed=true last_publish_ack_replayable=true last_publish_ack_index_status=healthy last_publish_ack_torn_tail_bytes=0\nparallel_replay_contract=parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope\n",
    )
    .expect("write transaction boundary without source dataset identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let transaction_boundary = summary
        .gates
        .iter()
        .find(|gate| gate.code == "transaction_boundary")
        .expect("transaction boundary gate");
    assert_eq!(
        transaction_boundary.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        transaction_boundary.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check transaction identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_transaction_boundary_without_source_ack_lsn() {
    let root = temp_root("pilot-evidence-check-source-ack");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50\ntarget_checkpoint last_durable_lsn=0/16B6C50\n",
    )
    .expect("write transaction boundary without source ack lsn");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let transaction_boundary = summary
        .gates
        .iter()
        .find(|gate| gate.code == "transaction_boundary")
        .expect("transaction boundary gate");
    assert_eq!(
        transaction_boundary.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(transaction_boundary
        .missing_markers
        .contains(&"source checkpoint evidence".to_string()));
    assert!(transaction_boundary
        .missing_markers
        .contains(&"target checkpoint evidence".to_string()));
    assert!(transaction_boundary
        .missing_markers
        .contains(&"source_ack_lsn evidence".to_string()));
    assert!(transaction_boundary
        .missing_markers
        .contains(&"source ack after durable publish".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check source ack temp dir");
}

#[test]
fn pilot_evidence_check_rejects_transaction_boundary_with_malformed_source_ack_lsn() {
    let root = temp_root("pilot-evidence-check-source-ack-lsn");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50\ntarget_checkpoint last_durable_lsn=0/16B6C50\nsource_ack_lsn=not-a-lsn\nsource_ack_after_durable_publish=true\nevery Trellara publish ack is durable\n",
    )
    .expect("write transaction boundary with malformed source ack lsn");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let transaction_boundary = summary
        .gates
        .iter()
        .find(|gate| gate.code == "transaction_boundary")
        .expect("transaction boundary gate");
    assert_eq!(
        transaction_boundary.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(transaction_boundary
        .missing_markers
        .contains(&"source_ack_lsn evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check source ack lsn temp dir");
}

#[test]
fn pilot_evidence_check_rejects_transaction_boundary_with_stale_target_checkpoint() {
    let root = temp_root("pilot-evidence-check-target-checkpoint-stale-lsn");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
        "source_id=local-source dataset_id=retail-sales\ntransaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B8000 last_durable_lsn=0/16B8000\ntarget_checkpoint last_durable_lsn=0/16B8000 last_applied_lsn=0/16B6C50\nsource_ack_lsn=0/16B8000\nsource_ack_after_durable_publish=true\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=1\nevery Trellara publish ack is durable\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_durability=fsync source_ack_crash_safe_ack=true source_ack_expected_publish_messages=1 source_ack_durable_publish_acks=1 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=1\nlast_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack last_publish_ack_durability=fsync last_publish_ack_crash_safe_ack=true last_publish_ack_topic=trellara.local-source.retail-sales.strict last_publish_ack_partition=0 last_publish_ack_offset=0 last_publish_ack_indexed=true last_publish_ack_replayable=true last_publish_ack_index_status=healthy last_publish_ack_torn_tail_bytes=0\nparallel_replay_contract=parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope\n",
    )
    .expect("write transaction boundary with stale target checkpoint");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);
    let transaction_boundary = summary
        .gates
        .iter()
        .find(|gate| gate.code == "transaction_boundary")
        .expect("transaction boundary gate");

    assert_eq!(
        transaction_boundary.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        transaction_boundary.missing_markers,
        vec!["target checkpoint evidence".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check stale target checkpoint temp dir");
}

#[test]
fn pilot_evidence_check_rejects_transaction_boundary_with_stale_source_ack_lsn() {
    let root = temp_root("pilot-evidence-check-source-ack-stale-lsn");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B8000 last_durable_lsn=0/16B8000\ntarget_checkpoint last_durable_lsn=0/16B8000 last_applied_lsn=0/16B8000\nsource_ack_lsn=0/16B6C50\nsource_ack_after_durable_publish=true\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=1\nevery Trellara publish ack is durable\n",
    )
    .expect("write transaction boundary with stale source ack lsn");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let transaction_boundary = summary
        .gates
        .iter()
        .find(|gate| gate.code == "transaction_boundary")
        .expect("transaction boundary gate");
    assert_eq!(
        transaction_boundary.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(transaction_boundary
        .missing_markers
        .contains(&"source_ack_lsn evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check stale source ack lsn temp dir");
}
