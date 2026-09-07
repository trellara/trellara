use super::*;

#[test]
fn pilot_evidence_check_accepts_gate_specific_artifacts() {
    let root = temp_root("pilot-evidence-check");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
            evidence_dir.join("source-safety.txt"),
            "Trellara source safety\nsource: local-source\ndataset: retail-sales\nstatus: degraded\ngrade: B (85/100)\n\nfindings:\n- [warning] source_slot_failover_disabled\n",
        )
        .expect("write source safety");
    fs::write(
        evidence_dir.join("contract-test.json"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "passed": true,
            "check_count": 2,
            "issue_count": 0,
            "recovery_action_count": 0,
            "checks": [
                {
                    "name": "source_and_target_contract:public.sales",
                    "passed": true,
                    "severity": "info",
                    "message": "public.sales satisfies source capture and target compatibility checks",
                    "recommendation": ""
                },
                {
                    "name": "transaction_boundary:strict",
                    "passed": true,
                    "severity": "info",
                    "message": "strict transaction boundary is configured before CDC starts",
                    "recommendation": ""
                }
            ],
            "recovery_actions": []
        }"#,
    )
    .expect("write contract");
    fs::write(
        evidence_dir.join("transaction-boundary.txt"),
            "source_id=local-source dataset_id=retail-sales\ntransaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50 last_durable_lsn=0/16B6C50\ntarget_checkpoint last_durable_lsn=0/16B6C50 last_applied_lsn=0/16B6C50\nsource_ack_lsn=0/16B6C50\nsource_ack_after_durable_publish=true\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=1\nevery Trellara publish ack is durable\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_durability=fsync source_ack_crash_safe_ack=true source_ack_expected_publish_messages=1 source_ack_durable_publish_acks=1 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=1\nlast_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack last_publish_ack_durability=fsync last_publish_ack_crash_safe_ack=true last_publish_ack_topic=trellara.local-source.retail-sales.strict last_publish_ack_partition=0 last_publish_ack_offset=0 last_publish_ack_indexed=true last_publish_ack_replayable=true last_publish_ack_index_status=healthy last_publish_ack_torn_tail_bytes=0\nparallel_replay_contract=parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=1\npartitioned_scale_envelope_event_count=1\npartitioned_scale_event_count_coverage=true\npartitioned_scale_participating_partition_ids=0\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition\n",
        )
        .expect("write transaction boundary");
    fs::write(
        evidence_dir.join("snapshot.txt"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "run_id": "pilot-snapshot-1",
            "state": "stream_handoff_ready",
            "slot": "trellara_retail_sales",
            "consistent_lsn": "0/16B6C50",
            "selected_table_count": 1,
            "table_count": 1,
            "skipped_table_count": 0,
            "copied_rows": 3,
            "tables": [
                {
                    "relation": "public.sales",
                    "state": "copy_complete",
                    "copied_rows": 3,
                    "skipped": false,
                    "watermark_lsn": "0/16B6C50"
                }
            ],
            "next_commands": ["trellara relay --config <config>", "trellara apply --config <config>", "trellara verify --config <config>"],
            "consistency_note": "copies source tables inside the exported logical snapshot held by the pgoutput replication connection, then hands off at the slot consistent LSN"
        }"#,
    )
    .expect("write snapshot");
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        "source_id=local-source dataset_id=retail-sales converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales target_relation=public.sales relation_match=true\n",
    )
    .expect("write verify");
    fs::write(
        evidence_dir.join("diagnostics.txt"),
        "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nalerts: 0\nrepair_plan_required: false\n\nattachment_commands:\n- trellara status --config trellara.yml --view diagnostics --format text\n- trellara repair-plan --config trellara.yml\n- trellara quarantine list --config trellara.yml\n\nrecommended_actions:\n- run trellara verify after repair; use trellara reseed if checksum drift remains\n",
    )
    .expect("write diagnostics");
    fs::write(
        evidence_dir.join("lake-writer-plan.json"),
        r#"{
            "dataset_id": "retail-sales",
            "duplicate_transaction_count": 1,
            "duplicate_replay_evidence": {
                "contract": "duplicate replay is skipped by transaction identity and conflicting idempotency evidence fails closed",
                "duplicate_transaction_count": 1,
                "unique_transaction_count": 1,
                "row_intent_count": 1,
                "idempotency_key_count": 1,
                "replay_safe": true
            },
            "committer_topology": {
                "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
            },
            "commit_steps": [
                {"order": 10, "phase": "raw_cdc_data", "durability_gate": "before source acknowledgement"},
                {"order": 120, "phase": "epoch_row_metadata", "durability_gate": "all planned raw CDC data files and Iceberg checkpoint receipts for the epoch are durable before this metadata step"},
                {"order": 130, "phase": "verification_metadata", "durability_gate": "all planned raw CDC data files and Iceberg checkpoint receipts for the epoch are durable before this metadata step"}
            ],
            "recovery_scenarios": [
                {"replay_policy": "discover_existing_epoch_metadata_before_publish", "recovery_action": "verify checkpoint receipts and discover existing epoch metadata before publishing duplicate visibility"}
            ],
            "epoch_metadata": {
                "source_rows": [
                    {"source_id": "local-source"}
                ],
                "partition_rows": [
                    {"source_id": "local-source", "partition_id": 0}
                ]
            },
            "row_intents": [
                {
                    "source_id": "local-source",
                    "dataset_id": "retail-sales",
                    "relation": "public.sales",
                    "transaction_id": "tx-lake-writer-sample",
                    "total_order": 2,
                    "idempotency_key": "local-source:0/16B6C50:tx-lake-writer-sample:2",
                    "schema_version": 67890,
                    "ddl_barrier_id": "local-source:retail-sales:tx-lake-writer-sample:0/16B6C50:ddl",
                    "ddl_release_gate": "post_ddl_dml_release",
                    "ddl_schema_fingerprint_before": 12345,
                    "ddl_schema_fingerprint_after": 67890
                }
            ]
        }"#,
    )
    .expect("write lake writer plan");
    fs::write(
        evidence_dir.join("lake-completeness.json"),
        r#"{
            "dataset_id": "retail-sales",
            "source_rows": [
                {"source_id": "local-source", "state": "complete"}
            ],
            "verification_status": "match",
            "spark_consumption_allowed": true,
            "source_counts_match": true,
            "stream_required_source_count": 1,
            "stream_complete_source_count": 1,
            "stream_missing_source_count": 0,
            "stream_quarantined_source_count": 0,
            "lake_required_source_count": 1,
            "lake_complete_source_count": 1,
            "lake_missing_source_count": 0,
            "lake_quarantined_source_count": 0,
            "checksum_rollup_match": true,
            "stream_checksum_rollup": 991,
            "lake_checksum_rollup": 991,
            "spark_consumption_contract": "lake_epoch_consumption_requires_matching_verified_metadata_and_explicit_gap_acceptance",
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#,
    )
    .expect("write lake completeness");
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "release_decision": {
                "release_dml": true,
                "blocker_codes": []
            },
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ],
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "propagation_boundary": "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks",
            "propagation_decisions": ["auto_apply:1", "manual_review:0", "unsupported:0", "target_ack_required:1"],
            "propagation_policy_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ],
            "ack_evidence": [
                {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "source_ack_lsn": "0/16B8000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ],
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "ddl_dml_replay_proof": {
                "contract": "target_postgres_applies_ddl_ack_then_releases_dml_at_same_cdc_boundary",
                "source_id": "local-source",
                "dataset_id": "retail-sales",
                "barrier_id": "ddl-barrier-abc",
                "barrier_lsn": "0/16B8000",
                "target_ack_lsn": "0/16B8000",
                "dml_commit_lsn": "00000000/016B8000",
                "dml_decision": "Applied",
                "ddl_applied_statements": 1,
                "dml_applied_changes": 1,
                "schema_version": "schema-v2",
                "release_gate": "post_ddl_dml_release",
                "target_transaction_boundary": "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release",
                "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "proof_steps": [
                    "target_postgres_recorded_ddl_ack",
                    "release_decision_allowed_post_ddl_dml",
                    "dml_replay_applied_changes_positive",
                    "dml_replay_commit_lsn_matches_ddl_barrier_lsn"
                ]
            },
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_eq!(summary.verdict, "live_evidence_accepted");
    assert_eq!(summary.accepted_gate_count, 9);
    assert_eq!(summary.missing_gate_count, 0);
    assert_eq!(summary.insufficient_gate_count, 0);
    assert!(summary
        .gates
        .iter()
        .any(|gate| gate.code == "transaction_boundary"
            && gate.evidence_status == PilotLiveEvidenceStatus::Accepted));
    assert!(summary
        .gates
        .iter()
        .any(|gate| gate.code == "verified_apply"
            && gate.evidence_status == PilotLiveEvidenceStatus::Accepted));
    assert!(summary
        .review_rule
        .contains("every needs_live_evidence gate"));

    fs::remove_dir_all(root).expect("remove evidence check temp dir");
}
