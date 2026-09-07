use super::*;

#[test]
fn pilot_evidence_check_rejects_ddl_release_proof_with_contradictory_blockers() {
    let root = temp_root("pilot-evidence-check-ddl-proof-contradictory-blockers");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("ddl-release-proof.json"),
        r#"{
            "release_decision": {
                "release_dml": true,
                "blocker_codes": ["pending_required_sink_ack"]
            },
            "release_gates": [
                {"release_gate_code": "post_ddl_dml_release", "satisfied": true}
            ],
            "cdc_transaction_boundary": "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn",
            "ack_commands": [
                "trellara schema ddl-barrier ack --config trellara.yml --sink raw_cdc_lake"
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
    .expect("write ddl release proof with contradictory blockers");
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
    assert!(ddl_release
        .missing_markers
        .contains(&"no release blockers".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"release_dml true".to_string()));
    assert!(ddl_release
        .missing_markers
        .contains(&"post_ddl_dml_release".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check contradictory blockers temp dir");
}
