use super::*;

mod ack;
mod blockers;
mod partition_visibility_ack;

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_without_ack_evidence() {
    let root = temp_root("pilot-evidence-check-ddl-proof");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
            "release_decision": {
                "release_dml": false,
                "blocker_codes": ["pending_required_sink_ack"]
            },
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": false}
            ],
            "ack_commands": [],
            "ack_evidence": [],
            "release_blocker_codes": ["pending_required_sink_ack"]
        }"#,
    )
    .expect("write ddl release proof without ack evidence");
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
        .contains(&"release_dml true".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_commands".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"ack_evidence".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"post_ddl_dml_release".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"release_evidence".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"no release blockers".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check ddl proof temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-ddl-proof-identity");
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
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
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
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof without source dataset identity");
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
    assert_eq!(
        ddl_release.missing_markers,
        vec![
            "source dataset identity".to_string(),
            "propagation_boundary".to_string(),
            "propagation_decisions".to_string(),
            "propagation_policy_sha256".to_string(),
            "ddl_dml_replay_proof".to_string(),
        ]
    );

    fs::remove_dir_all(root).expect("remove ddl release proof identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_unsatisfied_post_ddl_gate() {
    let root = temp_root("pilot-evidence-check-unsatisfied-ddl-release-gate");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
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
                {"release_gate_code": "post_ddl_dml_release", "satisfied": false}
            ],
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
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
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof with unsatisfied post ddl gate");
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
    assert_eq!(
        ddl_release.missing_markers,
        vec![
            "release_dml true".to_string(),
            "post_ddl_dml_release".to_string(),
            "propagation_boundary".to_string(),
            "propagation_decisions".to_string(),
            "propagation_policy_sha256".to_string(),
            "ddl_dml_replay_proof".to_string(),
        ]
    );

    fs::remove_dir_all(root).expect("remove unsatisfied ddl release gate temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_stale_release_evidence() {
    let root = temp_root("pilot-evidence-check-stale-ddl-release-evidence");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
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
            "barrier_lsn": "0/16B9000",
            "schema_version": "schema-v3",
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v3 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ],
            "ack_evidence": [
                {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "source_ack_lsn": "0/16B9000", "schema_version": "schema-v3", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v3 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
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
    .expect("write ddl release proof with stale release evidence");
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
    assert_eq!(
        ddl_release.missing_markers,
        vec![
            "propagation_boundary".to_string(),
            "propagation_decisions".to_string(),
            "propagation_policy_sha256".to_string(),
            "release_evidence".to_string(),
            "ddl_dml_replay_proof".to_string(),
        ]
    );

    fs::remove_dir_all(root).expect("remove stale ddl release evidence temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_wrong_required_sink_count() {
    let root = temp_root("pilot-evidence-check-wrong-ddl-release-ack-count");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
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
            "required_sinks": ["target_postgres", "raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink target_postgres --ack-lsn 0/16B9000 --schema-version schema-v2 --plan-sha256 aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --statement-sha256 bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ],
            "ack_evidence": [
                {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "target_postgres", "barrier_id": "ddl-barrier-abc", "source_ack_lsn": "0/16B8000", "schema_version": "schema-v2", "accepted": true, "detail": "target Postgres applied 1 DDL statements; plan_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; statement_sha256=bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb; release_gate=post_ddl_dml_release"},
                {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-abc", "source_ack_lsn": "0/16B8000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
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
    .expect("write ddl release proof with wrong required sink count");
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
    assert_eq!(
        ddl_release.missing_markers,
        vec![
            "propagation_boundary".to_string(),
            "propagation_decisions".to_string(),
            "propagation_policy_sha256".to_string(),
            "release_evidence".to_string(),
            "ddl_dml_replay_proof".to_string(),
        ]
    );

    fs::remove_dir_all(root).expect("remove wrong ddl release ack count temp dir");
}

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_wrong_barrier_id_summary() {
    let root = temp_root("pilot-evidence-check-wrong-ddl-release-barrier-id");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
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
            "barrier_id": "ddl-barrier-new",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["raw_cdc_lake"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-new --sink raw_cdc_lake --ack-lsn 0/16B9000 --schema-version schema-v2 --epoch-id epoch-1 --metadata-table _trellara_raw_cdc_epochs --partition-metadata-table _trellara_epoch_partitions --manifest-digest abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"
            ],
            "ack_evidence": [
                {"source_id": "local-source", "dataset_id": "retail-sales", "sink": "raw_cdc_lake", "barrier_id": "ddl-barrier-new", "source_ack_lsn": "0/16B8000", "schema_version": "schema-v2", "accepted": true, "detail": "raw CDC lake recorded schema_version schema-v2 for epoch epoch-1 in metadata table _trellara_raw_cdc_epochs; partition_metadata_table=_trellara_epoch_partitions; manifest_digest=abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789; release_gate=post_ddl_dml_release"}
            ],
            "release_evidence": [
                "barrier ddl-barrier-old recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "release_blocker_codes": []
        }"#,
    )
    .expect("write ddl release proof with wrong barrier id summary");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);
    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");

    assert_eq!(
        ddl_release.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        ddl_release.missing_markers,
        vec![
            "propagation_boundary".to_string(),
            "propagation_decisions".to_string(),
            "propagation_policy_sha256".to_string(),
            "release_evidence".to_string(),
            "ddl_dml_replay_proof".to_string(),
        ]
    );

    fs::remove_dir_all(root).expect("remove wrong ddl release barrier id temp dir");
}
