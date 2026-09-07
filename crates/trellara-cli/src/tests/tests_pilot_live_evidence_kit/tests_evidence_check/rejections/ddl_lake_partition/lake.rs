use super::*;

#[test]
fn pilot_evidence_check_rejects_lake_consumption_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-lake-missing-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("lake-completeness.json"),
        r#"{
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
    .expect("write lake completeness without identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let lake = summary
        .gates
        .iter()
        .find(|gate| gate.code == "lake_spark_consumption")
        .expect("lake spark consumption gate");
    assert_eq!(lake.evidence_status, PilotLiveEvidenceStatus::Insufficient);
    assert_eq!(
        lake.missing_markers,
        vec![
            "source dataset identity".to_string(),
            "source count agreement".to_string()
        ]
    );

    fs::remove_dir_all(root).expect("remove evidence check lake missing identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_lake_consumption_without_released_json_gate() {
    let root = temp_root("pilot-evidence-check-lake-consumption");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("lake-completeness.json"),
        r#"{
            "verification_status": "match",
            "spark_consumption_allowed": false,
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
            "spark_consumption_gate": "blocked: complete_with_gaps requires explicit acceptance",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#,
    )
    .expect("write blocked lake completeness");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let lake = summary
        .gates
        .iter()
        .find(|gate| gate.code == "lake_spark_consumption")
        .expect("lake spark consumption gate");
    assert_eq!(lake.evidence_status, PilotLiveEvidenceStatus::Insufficient);
    assert!(lake
        .missing_markers
        .contains(&"spark_consumption_allowed true".to_string()));
    assert!(lake
        .missing_markers
        .contains(&"released spark consumption gate".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check lake consumption temp dir");
}

#[test]
fn pilot_evidence_check_rejects_lake_consumption_with_released_gate_but_mismatch_status() {
    let root = temp_root("pilot-evidence-check-lake-released-mismatch");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("lake-completeness.json"),
        r#"{
            "verification_status": "mismatch",
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
    .expect("write contradictory lake completeness");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let lake = summary
        .gates
        .iter()
        .find(|gate| gate.code == "lake_spark_consumption")
        .expect("lake spark consumption gate");
    assert_eq!(lake.evidence_status, PilotLiveEvidenceStatus::Insufficient);
    assert!(lake
        .missing_markers
        .contains(&"verification_status match".to_string()));
    assert!(lake
        .missing_markers
        .contains(&"released spark consumption gate".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check lake released mismatch temp dir");
}

#[test]
fn pilot_evidence_check_rejects_lake_consumption_without_durable_source_ack_boundary() {
    let root = temp_root("pilot-evidence-check-lake-source-ack");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("lake-completeness.json"),
        r#"{
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
            "source_ack_boundary": "operator reviewed source ack behavior"
        }"#,
    )
    .expect("write weak lake completeness");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let lake = summary
        .gates
        .iter()
        .find(|gate| gate.code == "lake_spark_consumption")
        .expect("lake spark consumption gate");
    assert_eq!(lake.evidence_status, PilotLiveEvidenceStatus::Insufficient);
    assert!(lake
        .missing_markers
        .contains(&"source ack boundary".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check lake source ack temp dir");
}

#[test]
fn pilot_evidence_check_rejects_lake_consumption_with_vague_release_gate() {
    let root = temp_root("pilot-evidence-check-lake-vague-release");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("lake-completeness.json"),
        r#"{
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
            "spark_consumption_gate": "released by operator",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#,
    )
    .expect("write vague lake completeness");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let lake = summary
        .gates
        .iter()
        .find(|gate| gate.code == "lake_spark_consumption")
        .expect("lake spark consumption gate");
    assert_eq!(lake.evidence_status, PilotLiveEvidenceStatus::Insufficient);
    assert!(lake
        .missing_markers
        .contains(&"released spark consumption gate".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check lake vague release temp dir");
}

#[test]
fn pilot_evidence_check_rejects_lake_consumption_without_contract() {
    let root = temp_root("pilot-evidence-check-lake-contract");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
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
            "spark_consumption_gate": "released: stream and lake proofs match",
            "source_ack_boundary": "source acknowledgement advances after durable Trellara stream publish, not after Iceberg catalog commit"
        }"#,
    )
    .expect("write lake completeness without contract");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let lake = summary
        .gates
        .iter()
        .find(|gate| gate.code == "lake_spark_consumption")
        .expect("lake spark consumption gate");
    assert_eq!(lake.evidence_status, PilotLiveEvidenceStatus::Insufficient);
    assert!(lake
        .missing_markers
        .contains(&"spark consumption contract".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check lake contract temp dir");
}
