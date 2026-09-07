use super::*;

#[test]
fn pilot_evidence_check_accepts_partition_visibility_ack_with_watermark_detail() {
    let root = temp_root("pilot-evidence-check-partition-visibility-ack-valid");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    write_partition_visibility_proof(&evidence_dir, valid_partition_visibility_detail());
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");
    assert_eq!(
        ddl_release.evidence_status,
        PilotLiveEvidenceStatus::Accepted
    );
    assert!(!ddl_release
        .missing_markers
        .contains(&"ack_evidence".to_string()));
    fs::remove_dir_all(root).expect("remove partition visibility valid ack temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_visibility_ack_without_watermark_detail() {
    let root = temp_root("pilot-evidence-check-partition-visibility-ack-missing-detail");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    write_partition_visibility_proof(&evidence_dir, "");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ddl_ack_evidence_insufficient(summary);
    fs::remove_dir_all(root).expect("remove partition visibility missing detail temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_visibility_ack_with_lagging_durable_lsn() {
    let root = temp_root("pilot-evidence-check-partition-visibility-ack-lagging-durable");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    write_partition_visibility_proof(
        &evidence_dir,
        "partition visibility reached barrier with 3/3 partitions at global_durable_lsn 0/16B7000 and global_applied_lsn 0/16B9000; partition_watermark_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; release_gate=post_ddl_dml_release",
    );
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ddl_ack_evidence_insufficient(summary);
    fs::remove_dir_all(root).expect("remove partition visibility lagging durable temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_visibility_ack_with_applied_lsn_mismatch() {
    let root = temp_root("pilot-evidence-check-partition-visibility-ack-applied-mismatch");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    write_partition_visibility_proof(
        &evidence_dir,
        "partition visibility reached barrier with 3/3 partitions at global_durable_lsn 0/16B9000 and global_applied_lsn 0/16B8000; partition_watermark_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; release_gate=post_ddl_dml_release",
    );
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ddl_ack_evidence_insufficient(summary);
    fs::remove_dir_all(root).expect("remove partition visibility applied mismatch temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_visibility_ack_command_without_partition_lsns() {
    let root = temp_root("pilot-evidence-check-partition-visibility-ack-command-missing-lsns");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    write_partition_visibility_proof_with_command(
        &evidence_dir,
        "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink partition_visibility --ack-lsn 0/16B9000 --schema-version schema-v2 --detail partition-visibility-evidence",
        valid_partition_visibility_detail(),
        &format!(
            "partition_visibility release evidence for ddl-barrier-abc: {}",
            valid_partition_visibility_detail()
        ),
    );
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ddl_ack_commands_insufficient(summary);
    fs::remove_dir_all(root).expect("remove partition visibility command missing lsns temp dir");
}

#[test]
fn pilot_evidence_check_rejects_partition_release_evidence_without_digest() {
    let root = temp_root("pilot-evidence-check-partition-visibility-release-digest");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    write_partition_visibility_proof_with_release_evidence(
        &evidence_dir,
        valid_partition_visibility_detail(),
        "partition_visibility release evidence for ddl-barrier-abc: partition visibility reached barrier with 3/3 partitions at global_durable_lsn 0/16B9000 and global_applied_lsn 0/16B9000; release_gate=post_ddl_dml_release",
    );
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let ddl_release = summary
        .gates
        .iter()
        .find(|gate| gate.code == "ddl_release_proof")
        .expect("ddl release proof gate");
    assert!(ddl_release
        .missing_markers
        .contains(&"release_evidence".to_string()));
    fs::remove_dir_all(root).expect("remove partition visibility missing release digest temp dir");
}

fn valid_partition_visibility_detail() -> &'static str {
    "partition visibility reached barrier with 3/3 partitions at global_durable_lsn 0/16B9000 and global_applied_lsn 0/16B9000; partition_watermark_sha256=aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa; release_gate=post_ddl_dml_release"
}

fn write_partition_visibility_proof(evidence_dir: &Path, detail: &str) {
    write_partition_visibility_proof_with_release_evidence(
        evidence_dir,
        detail,
        &format!("partition_visibility release evidence for ddl-barrier-abc: {detail}"),
    );
}

fn write_partition_visibility_proof_with_release_evidence(
    evidence_dir: &Path,
    detail: &str,
    release_evidence: &str,
) {
    write_partition_visibility_proof_with_command(
        evidence_dir,
        "trellara schema ddl-barrier ack --config trellara.yml --barrier-id ddl-barrier-abc --sink partition_visibility --ack-lsn 0/16B9000 --schema-version schema-v2 --barrier-lsn 0/16B8000 --expected-partition-count 3 --partition-durable-lsn 0=0/16B9000 --partition-applied-lsn 0=0/16B9000 --partition-durable-lsn 1=0/16B9000 --partition-applied-lsn 1=0/16B9000 --partition-durable-lsn 2=0/16B9000 --partition-applied-lsn 2=0/16B9000",
        detail,
        release_evidence,
    );
}

fn write_partition_visibility_proof_with_command(
    evidence_dir: &Path,
    command: &str,
    detail: &str,
    release_evidence: &str,
) {
    let detail_field = if detail.is_empty() {
        String::new()
    } else {
        format!(r#", "detail": "{detail}""#)
    };
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        format!(
            r#"{{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "release_decision": {{"release_dml": true, "blocker_codes": []}},
            "release_gates": [
                {{"release_gate_code": "post_ddl_dml_release", "satisfied": true}}
            ],
            "barrier_id": "ddl-barrier-abc",
            "barrier_lsn": "0/16B8000",
            "schema_version": "schema-v2",
            "required_sinks": ["partition_visibility"],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "propagation_boundary": "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks",
            "propagation_decisions": ["auto_apply:1", "manual_review:0", "unsupported:0", "target_ack_required:1"],
            "propagation_policy_sha256": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "ack_commands": [
                "{command}"
            ],
            "ack_evidence": [
                {{"source_id": "local-source", "dataset_id": "retail-sales", "sink": "partition_visibility", "barrier_id": "ddl-barrier-abc", "ack_lsn": "0/16B9000", "schema_version": "schema-v2", "accepted": true{detail_field}}}
            ],
            "release_evidence": [
                "barrier ddl-barrier-abc recorded at lsn 0/16B8000 with schema_version schema-v2",
                "cdc_transaction_boundary: source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
                "1/1 required sink ACKs accepted",
                "{release_evidence}",
                "post-DDL DML release_dml=true blocker_codes=none"
            ],
            "ddl_dml_replay_proof": {{
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
            }},
            "release_blocker_codes": []
        }}"#
        ),
    )
    .expect("write partition visibility ddl release proof");
}

fn assert_ddl_ack_commands_insufficient(summary: PilotLiveEvidenceCheckSummary) {
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
}

fn assert_ddl_ack_evidence_insufficient(summary: PilotLiveEvidenceCheckSummary) {
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
}
