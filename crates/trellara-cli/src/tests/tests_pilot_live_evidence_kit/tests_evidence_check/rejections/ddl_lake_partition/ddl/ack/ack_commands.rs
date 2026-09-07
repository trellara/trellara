use super::*;

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_stale_ack_command_lsn() {
    let root = temp_root("pilot-evidence-check-ddl-proof-stale-ack-command-lsn");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
            "source_id": "source-a",
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
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B7000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ],
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ],
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof with stale ack command lsn");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");
    assert_eq!(
        ddl_release.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_commands".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check stale ack command lsn temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_unsupported_sink_ack() {
    let root = temp_root("pilot-evidence-check-ddl-proof-unsupported-sink");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
            "source_id": "source-a",
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
            "required_sinks": ["warehouse_projection"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink warehouse_projection --ack-lsn 0/16B9000 --schema-version schema-v2"
            ],
            "ack_evidence": [
                {"sink": "warehouse_projection", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true, "detail": "warehouse projection accepted schema_version schema-v2; release_gate=post_ddl_dml_release"}
            ],
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof with unsupported sink ack");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");
    assert_eq!(
        ddl_release.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_commands".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check unsupported sink temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_missing_required_sink_ack_command() {
    let root = temp_root("pilot-evidence-check-ddl-proof-missing-required-sink-command");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
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
            "required_sinks": ["target_postgres", "raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2"
            ],
            "ack_evidence": [
                {"sink": "target_postgres", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true},
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true}
            ],
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "2/2 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof missing required sink ack command");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");
    assert_eq!(
        ddl_release.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_commands".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check missing sink command temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_placeholder_ack_command() {
    let root = temp_root("pilot-evidence-check-ddl-proof-placeholder-ack-command");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
            "release_decision": {
                "release_dml": true,
                "blocker_codes": []
            },
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "operator will acknowledge sinks"
            ],
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "source_ack_lsn": "0/16B6C50"}
            ],
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "4/4 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof with placeholder ack command");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");
    assert_eq!(
        ddl_release.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_commands".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check placeholder ack command temp dir");
}
