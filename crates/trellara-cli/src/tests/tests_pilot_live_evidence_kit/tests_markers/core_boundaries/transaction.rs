use super::*;

#[test]
fn transaction_boundary_requires_concrete_checkpoint_and_ack_evidence() {
    let proof = "source_id=local-source dataset_id=retail-sales\ntransaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50 last_durable_lsn=0/16B6C50 durable=true\ntarget_checkpoint last_durable_lsn=0/16B6C50 last_applied_lsn=0/16B6C50 caught_up=true\nsource_ack_lsn=0/16B6C50\nsource_ack_after_durable_publish=true\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=2\nevery Trellara publish ack is durable\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_durability=fsync source_ack_crash_safe_ack=true source_ack_expected_publish_messages=2 source_ack_durable_publish_acks=2 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=2\nlast_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack last_publish_ack_durability=fsync last_publish_ack_crash_safe_ack=true last_publish_ack_topic=trellara.local-source.retail-sales.strict last_publish_ack_partition=0 last_publish_ack_offset=1 last_publish_ack_indexed=true last_publish_ack_replayable=true last_publish_ack_index_status=healthy last_publish_ack_torn_tail_bytes=0\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=4\npartitioned_scale_event_count_coverage=true\nparticipating_partition_count=2\npartitioned_scale_participating_partition_ids=0,2\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition\n";
    let json_proof = r#"{
        "source_id": "local-source",
        "dataset_id": "retail-sales",
        "transaction_boundary": {
            "status": "verified",
            "source_checkpoint": {
                "last_seen_lsn": "0/16B6C50",
                "last_durable_lsn": "0/16B6C50"
            },
            "target_checkpoint": {
                "last_durable_lsn": "0/16B6C50",
                "last_applied_lsn": "0/16B6C50"
            },
            "source_ack_lsn": "0/16B6C50",
            "source_ack_after_durable_publish": true,
            "source_ack_durability_proof": {
                "contract": "every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack",
                "durability": "fsync",
                "crash_safe_ack": true,
                "expected_publish_messages": 2,
                "durable_publish_acks": 2,
                "all_publish_acks_proven": true,
                "proofed_ack_count": 2
            },
            "last_publish_ack_proof": {
                "contract": "local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack",
                "durability": "fsync",
                "crash_safe_ack": true,
                "topic": "trellara.local-source.retail-sales.strict",
                "partition": 0,
                "offset": 1,
                "indexed": true,
                "replayable": true,
                "index_status": "healthy",
                "torn_tail_bytes": 0
            },
            "source_ack_publish_destinations_match": true,
            "source_ack_publish_destination_count": 2,
            "parallel_replay_contract": "partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path",
            "partitioned_scale_manifest_checksum": 12345,
            "partitioned_scale_manifest_event_count": 4,
            "partitioned_scale_envelope_event_count": 4,
            "partitioned_scale_event_count_coverage": true,
            "participating_partition_count": 2,
            "partitioned_scale_participating_partition_ids": [0, 2],
            "partitioned_scale_visibility_contract": "global visibility waits for manifest, commit marker, and every participating partition"
        }
    }"#;

    for marker in live_evidence_expected_markers("transaction_boundary") {
        assert!(
            live_evidence_marker_present("transaction_boundary", &marker, proof),
            "expected marker {marker}"
        );
        assert!(
            live_evidence_marker_present("transaction_boundary", &marker, json_proof),
            "expected JSON marker {marker}"
        );
    }

    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source dataset identity",
        "dataset_id=retail-sales\ntransaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50 last_durable_lsn=0/16B6C50\ntarget_checkpoint last_durable_lsn=0/16B6C50 last_applied_lsn=0/16B6C50\nsource_ack_lsn=0/16B6C50\nsource_ack_after_durable_publish=true\nparallel_replay_contract=parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source dataset identity",
        r#"{
            "transaction_boundary": {
                "source_id": "local-source",
                "dataset_id": "missing",
                "status": "verified",
                "parallel_replay_contract": "parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source checkpoint evidence",
        "transaction_boundary: verified\nsource_checkpoint missing\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source checkpoint evidence",
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=not-a-lsn\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source checkpoint evidence",
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "target checkpoint evidence",
        "transaction_boundary: verified\ntarget_checkpoint last_durable_lsn=0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "target checkpoint evidence",
        "transaction_boundary: verified\ntarget_checkpoint last_durable_lsn=0/16B8000 last_applied_lsn=0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "target checkpoint evidence",
        r#"{
            "transaction_boundary": {
                "status": "verified",
                "target_checkpoint": {
                    "last_durable_lsn": "0/16B8000",
                    "last_applied_lsn": "0/16B6C50"
                }
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "target checkpoint evidence",
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B8000 last_durable_lsn=0/16B8000\ntarget_checkpoint last_durable_lsn=0/16B6C50 last_applied_lsn=0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "target checkpoint evidence",
        r#"{
            "transaction_boundary": {
                "status": "verified",
                "source_checkpoint": {
                    "last_seen_lsn": "0/16B8000",
                    "last_durable_lsn": "0/16B8000"
                },
                "target_checkpoint": {
                    "last_durable_lsn": "0/16B6C50",
                    "last_applied_lsn": "0/16B6C50"
                }
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source_ack_lsn evidence",
        "transaction_boundary: verified\nsource_ack_lsn evidence\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source_ack_lsn evidence",
        "transaction_boundary: verified\nsource_ack_lsn=0/0\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source_ack_lsn evidence",
        "transaction_boundary: verified\nsource_ack_lsn=0/16B8000\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source_ack_lsn evidence",
        r#"{
            "transaction_boundary": {
                "status": "verified",
                "source_ack_lsn": "0/16B8000"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source_ack_lsn evidence",
        "transaction_boundary: verified\nsource_checkpoint last_seen_lsn=0/16B8000 last_durable_lsn=0/16B8000\nsource_ack_lsn=0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source_ack_lsn evidence",
        r#"{
            "transaction_boundary": {
                "status": "verified",
                "source_checkpoint": {
                    "last_seen_lsn": "0/16B8000",
                    "last_durable_lsn": "0/16B8000"
                },
                "source_ack_lsn": "0/16B6C50"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack after durable publish",
        "transaction_boundary: verified\nsource_ack_after_durable_publish=false\nsource_ack_lsn=0/16B6C50\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack after durable publish",
        "transaction_boundary: verified\nsource_ack_after_durable_publish=true\nsource_ack_lsn=0/16B6C50\nevery Trellara publish ack is durable\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack after durable publish",
        "transaction_boundary: verified\nsource_ack_after_durable_publish=true\nevery Trellara publish ack is durable\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_expected_publish_messages=2 source_ack_durable_publish_acks=1 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=2\nlast_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack last_publish_ack_indexed=true last_publish_ack_replayable=true last_publish_ack_torn_tail_bytes=0\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack after durable publish",
        r#"{
            "transaction_boundary": {
                "source_ack_after_durable_publish": true,
                "source_ack_durability_proof": {
                    "contract": "every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack",
                    "expected_publish_messages": 2,
                    "durable_publish_acks": 2,
                    "all_publish_acks_proven": false,
                    "proofed_ack_count": 2
                },
                "last_publish_ack_proof": {
                    "contract": "local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack",
                    "indexed": true,
                    "replayable": true,
                    "torn_tail_bytes": 0
                }
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack after durable publish",
        "transaction_boundary: verified\nsource_ack_after_durable_publish=true\nevery Trellara publish ack is durable\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_durability=buffered source_ack_crash_safe_ack=false source_ack_expected_publish_messages=1 source_ack_durable_publish_acks=1 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=1\nlast_publish_ack_proof=local_publish_ack_is_indexed_replayable_and_untorn_before_source_ack last_publish_ack_durability=buffered last_publish_ack_crash_safe_ack=false last_publish_ack_indexed=true last_publish_ack_replayable=true last_publish_ack_torn_tail_bytes=0\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack publish destinations",
        "transaction_boundary: verified\nsource_ack_publish_destinations_match=false source_ack_publish_destination_count=1\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack publish destinations",
        "transaction_boundary: verified\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=0\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack publish destinations",
        r#"{
            "transaction_boundary": {
                "source_ack_publish_destinations_match": true,
                "source_ack_publish_destination_count": 0
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack publish destinations",
        "transaction_boundary: verified\nsource_ack_publish_destinations_match=true source_ack_publish_destination_count=2\nsource_ack_durability_proof=every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack source_ack_expected_publish_messages=3 source_ack_durable_publish_acks=3 source_ack_all_publish_acks_proven=true source_ack_proofed_ack_count=3\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "source ack publish destinations",
        r#"{
            "transaction_boundary": {
                "source_ack_publish_destinations_match": true,
                "source_ack_publish_destination_count": 2,
                "source_ack_durability_proof": {
                    "contract": "every_local_publish_ack_is_indexed_replayable_untorn_and_destination_matched_before_source_ack",
                    "expected_publish_messages": 3,
                    "durable_publish_acks": 3,
                    "all_publish_acks_proven": true,
                    "proofed_ack_count": 3
                }
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "parallel replay contract",
        "transaction_boundary: verified\nparallel_replay_contract=DDL only\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=4\npartitioned_scale_event_count_coverage=true\npartitioned_scale_participating_partition_ids=0,not-a-partition\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=4\npartitioned_scale_event_count_coverage=true\nparticipating_partition_count=3\npartitioned_scale_participating_partition_ids=0,2\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=4\npartitioned_scale_event_count_coverage=true\npartitioned_scale_participating_partition_ids=0,0\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=3\npartitioned_scale_event_count_coverage=true\npartitioned_scale_participating_partition_ids=0,2\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=4\npartitioned_scale_event_count_coverage=true\npartitioned_scale_participating_partition_ids=0,2\npartitioned_scale_visibility_contract=global visibility waits for manifest and commit marker\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path\npartitioned_scale_manifest_checksum=12345\npartitioned_scale_manifest_event_count=4\npartitioned_scale_envelope_event_count=4\npartitioned_scale_event_count_coverage=true\npartitioned_scale_participating_partition_ids=0,2\npartitioned_scale_visibility_contract=global visibility waits for manifest, commit marker, and every participating partition with operator override\n"
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        r#"{
            "transaction_boundary": {
                "parallel_replay_contract": "partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path",
                "partitioned_scale_manifest_checksum": 12345,
                "partitioned_scale_manifest_event_count": 4,
                "partitioned_scale_envelope_event_count": 4,
                "partitioned_scale_event_count_coverage": true,
                "participating_partition_count": 3,
                "partitioned_scale_participating_partition_ids": [0, 2],
                "partitioned_scale_visibility_contract": "global visibility waits for manifest, commit marker, and every participating partition"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        r#"{
            "transaction_boundary": {
                "parallel_replay_contract": "partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path",
                "partitioned_scale_manifest_checksum": 12345,
                "partitioned_scale_manifest_event_count": 4,
                "partitioned_scale_envelope_event_count": 4,
                "partitioned_scale_event_count_coverage": true,
                "partitioned_scale_participating_partition_ids": ["0", "2"],
                "partitioned_scale_visibility_contract": "global visibility waits for manifest, commit marker, and every participating partition"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        r#"{
            "transaction_boundary": {
                "parallel_replay_contract": "partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path",
                "partitioned_scale_manifest_checksum": 12345,
                "partitioned_scale_manifest_event_count": 4,
                "partitioned_scale_envelope_event_count": 4,
                "partitioned_scale_event_count_coverage": true,
                "partitioned_scale_participating_partition_ids": [0, 0],
                "partitioned_scale_visibility_contract": "global visibility waits for manifest, commit marker, and every participating partition"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        r#"{
            "transaction_boundary": {
                "parallel_replay_contract": "partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path",
                "partitioned_scale_manifest_checksum": 12345,
                "partitioned_scale_manifest_event_count": 4,
                "partitioned_scale_envelope_event_count": 3,
                "partitioned_scale_event_count_coverage": true,
                "partitioned_scale_participating_partition_ids": [0, 2],
                "partitioned_scale_visibility_contract": "global visibility waits for manifest, commit marker, and every participating partition"
            }
        }"#
    ));
    assert!(!live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        r#"{
            "transaction_boundary": {
                "parallel_replay_contract": "partition workers may replay DML-only committed transactions by partition; DDL-bearing transactions use the DDL barrier path",
                "partitioned_scale_manifest_checksum": 12345,
                "partitioned_scale_manifest_event_count": 4,
                "partitioned_scale_envelope_event_count": 4,
                "partitioned_scale_event_count_coverage": true,
                "partitioned_scale_participating_partition_ids": [0, 2],
                "partitioned_scale_visibility_contract": "global visibility waits for manifest, commit marker, and every participating partition with operator override"
            }
        }"#
    ));
    assert!(live_evidence_marker_present(
        "transaction_boundary",
        "partitioned manifest evidence headers",
        "transaction_boundary: verified\nparallel_replay_contract=parallel replay is disabled; each committed source transaction applies as one ordered atomic envelope\n"
    ));
}
