use super::*;

mod ack_commands;

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_malformed_ack_lsn() {
    let root = temp_root("pilot-evidence-check-ddl-proof-bad-ack-lsn");
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
                "trellara schema ddl-barrier ack --config trellara.yml --sink raw_cdc_lake"
            ],
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "source_ack_lsn": "not-a-lsn"}
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
    .expect("write ddl release proof with malformed ack lsn");
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
        .contains(&"ack_evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check bad ack lsn temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_ack_lsn_before_barrier() {
    let root = temp_root("pilot-evidence-check-ddl-proof-stale-ack-lsn");
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
            "barrier_lsn": "0/16B8000",
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --sink raw_cdc_lake"
            ],
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B6C50", "accepted": true}
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
    .expect("write ddl release proof with ack lsn before barrier");
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
        .contains(&"ack_evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check stale ack lsn temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_ack_schema_version_mismatch() {
    let root = temp_root("pilot-evidence-check-ddl-proof-ack-schema-version-mismatch");
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
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --sink raw_cdc_lake"
            ],
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v1", "accepted": true}
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
    .expect("write ddl release proof with ack schema version mismatch");
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
        .contains(&"ack_evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check ack schema mismatch temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_missing_required_sink_ack() {
    let root = temp_root("pilot-evidence-check-ddl-proof-missing-required-sink-ack");
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
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["target_postgres", "raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --sink target_postgres",
                "trellara schema ddl-barrier ack --config trellara.yml --sink raw_cdc_lake"
            ],
            "ack_evidence": [
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
    .expect("write ddl release proof missing required sink ack");
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
        .contains(&"ack_evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check missing required sink ack temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_ack_without_explicit_acceptance() {
    let root = temp_root("pilot-evidence-check-ddl-proof-ack-missing-accepted");
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
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2"
            ],
            "ack_evidence": [
                {"sink": "raw_cdc_lake", "ack_lsn": "0/16B9000", "schema_version": "schema-v2"}
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
    .expect("write ddl release proof ack missing accepted");
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
        .contains(&"ack_evidence".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check missing accepted temp dir");
}
