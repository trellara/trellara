use super::*;

#[test]
fn pilot_evidence_check_rejects_json_transaction_boundary_with_stale_source_ack_lsn() {
    let root = temp_root("pilot-evidence-check-json-source-ack-stale-lsn");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "transaction_boundary": {
                "status": "verified",
                "source_checkpoint": {
                    "last_seen_lsn": "0/16B8000",
                    "last_durable_lsn": "0/16B8000"
                },
                "target_checkpoint": {
                    "last_durable_lsn": "0/16B8000",
                    "last_applied_lsn": "0/16B8000"
                },
                "source_ack_lsn": "0/16B6C50",
                "source_ack_after_durable_publish": true,
                "source_ack_durability_proof": {
                    "contract": "every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack",
                    "durability": "fsync",
                    "crash_safe_ack": true,
                    "expected_publish_messages": 1,
                    "durable_publish_acks": 1,
                    "all_publish_acks_proven": true,
                    "proofed_ack_count": 1
                },
                "last_publish_ack_proof": {
                    "contract": "local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack",
                    "durability": "fsync",
                    "crash_safe_ack": true,
                    "topic": "trellara.local-source.retail-sales.strict",
                    "partition": 0,
                    "offset": 0,
                    "indexed": true,
                    "replayable": true,
                    "index_status": "healthy",
                    "torn_tail_bytes": 0
                },
                "source_ack_publish_destinations_match": true,
                "source_ack_publish_destination_count": 1,
                "parallel_replay_contract": "parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope"
            }
        }"#,
    )
    .expect("write JSON transaction boundary with stale source ack lsn");
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
        vec!["source_ack_lsn evidence".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check JSON stale source ack lsn temp dir");
}
