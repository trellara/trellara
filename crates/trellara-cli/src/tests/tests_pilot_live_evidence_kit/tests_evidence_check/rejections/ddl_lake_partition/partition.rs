use super::*;

#[test]
fn pilot_evidence_check_rejects_partition_watermarks_without_global_boundary() {
    let root = temp_root("pilot-evidence-check-partition-watermarks");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-watermarks.json"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-east",
            "expected_partition_count": 2,
            "observed_partition_count": 2,
            "complete_partition_set": true,
            "global_durable_lsn": null,
            "global_applied_lsn": "0/16B9000",
            "missing_partitions": [],
            "partitions": [
                {
                    "partition_id": 0,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "durable_to_applied_bytes": 0,
                    "blocks_global_applied_watermark": false
                },
                {
                    "partition_id": 1,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "durable_to_applied_bytes": 0,
                    "blocks_global_applied_watermark": false
                }
            ]
        }"#,
    )
    .expect("write partition watermarks without global durable boundary");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let partition = summary
        .gates
        .iter()
        .find(|gate| gate.code == "partition_watermarks")
        .expect("partition watermarks gate");
    assert_eq!(
        partition.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(partition
        .missing_markers
        .contains(&"complete_partition_set true".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check partition watermarks temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_watermarks_without_artifact_identity() {
    let root = temp_root("pilot-evidence-check-partition-watermarks-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-watermarks.json"),
        r#"{
            "expected_partition_count": 2,
            "observed_partition_count": 2,
            "complete_partition_set": true,
            "global_durable_lsn": "0/16B9000",
            "global_applied_lsn": "0/16B9000",
            "missing_partitions": [],
            "partitions": [
                {
                    "partition_id": 0,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "blocks_global_applied_watermark": false
                },
                {
                    "partition_id": 1,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "blocks_global_applied_watermark": false
                }
            ]
        }"#,
    )
    .expect("write anonymous partition watermarks");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let partition = summary
        .gates
        .iter()
        .find(|gate| gate.code == "partition_watermarks")
        .expect("partition watermarks gate");
    assert_eq!(
        partition.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(partition
        .missing_markers
        .contains(&"source dataset identity".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check partition identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_watermarks_with_applied_ahead_of_durable() {
    let root = temp_root("pilot-evidence-check-partition-lsn-order");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-watermarks.json"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-east",
            "expected_partition_count": 2,
            "observed_partition_count": 2,
            "complete_partition_set": true,
            "global_durable_lsn": "0/16B8000",
            "global_applied_lsn": "0/16B9000",
            "missing_partitions": [],
            "partitions": [
                {
                    "partition_id": 0,
                    "last_durable_lsn": "0/16B8000",
                    "last_applied_lsn": "0/16B9000",
                    "durable_to_applied_bytes": 0,
                    "blocks_global_applied_watermark": false
                },
                {
                    "partition_id": 1,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "durable_to_applied_bytes": 0,
                    "blocks_global_applied_watermark": false
                }
            ]
        }"#,
    )
    .expect("write partition watermarks with invalid LSN order");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let partition = summary
        .gates
        .iter()
        .find(|gate| gate.code == "partition_watermarks")
        .expect("partition watermarks gate");
    assert_eq!(
        partition.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(partition
        .missing_markers
        .contains(&"complete_partition_set true".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check partition lsn order temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_watermarks_with_duplicate_partition_ids() {
    let root = temp_root("pilot-evidence-check-partition-duplicate-ids");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-watermarks.json"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-east",
            "expected_partition_count": 2,
            "observed_partition_count": 2,
            "complete_partition_set": true,
            "global_durable_lsn": "0/16B9000",
            "global_applied_lsn": "0/16B9000",
            "missing_partitions": [],
            "partitions": [
                {
                    "partition_id": 0,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "blocks_global_applied_watermark": false
                },
                {
                    "partition_id": 0,
                    "last_durable_lsn": "0/16B9000",
                    "last_applied_lsn": "0/16B9000",
                    "blocks_global_applied_watermark": false
                }
            ]
        }"#,
    )
    .expect("write partition watermarks with duplicate partition ids");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let partition = summary
        .gates
        .iter()
        .find(|gate| gate.code == "partition_watermarks")
        .expect("partition watermarks gate");
    assert_eq!(
        partition.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(partition
        .missing_markers
        .contains(&"complete_partition_set true".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check partition duplicate ids temp dir");
}
