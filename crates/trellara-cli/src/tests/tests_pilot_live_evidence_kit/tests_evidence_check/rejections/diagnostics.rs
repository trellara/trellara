use super::*;

#[test]
fn pilot_evidence_check_rejects_failure_drill_without_operational_context() {
    let root = temp_root("pilot-evidence-check-failure-drill-thin");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("diagnostics.txt"),
        "Trellara diagnostics\nrepair_plan_required: true\nquarantine list captured\n",
    )
    .expect("write thin diagnostics");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let failure_drill = summary
        .gates
        .iter()
        .find(|gate| gate.code == "failure_drill")
        .expect("failure drill gate");
    assert_eq!(
        failure_drill.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(failure_drill
        .missing_markers
        .contains(&"Trellara diagnostics".to_string()));
    assert!(failure_drill
        .missing_markers
        .contains(&"repair_plan_required".to_string()));
    assert!(failure_drill
        .missing_markers
        .contains(&"quarantine".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check failure drill temp dir");
}

#[test]
fn pilot_evidence_check_rejects_failure_drill_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-failure-drill-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("diagnostics.txt"),
        "Trellara diagnostics\nconfig: trellara.yml\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nalerts: 0\nrepair_plan_required: false\n\nattachment_commands:\n- trellara status --config trellara.yml --view diagnostics --format text\n- trellara repair-plan --config trellara.yml\n- trellara quarantine list --config trellara.yml\n\nrecommended_actions:\n- run trellara verify after repair\n",
    )
    .expect("write diagnostics without identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let failure_drill = summary
        .gates
        .iter()
        .find(|gate| gate.code == "failure_drill")
        .expect("failure drill gate");
    assert_eq!(
        failure_drill.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        failure_drill.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check failure drill identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_failure_drill_with_generic_repair_action() {
    let root = temp_root("pilot-evidence-check-failure-drill-generic-repair");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("diagnostics.txt"),
        "Trellara diagnostics\nconfig: trellara.yml\nsource: local-source\ndataset: retail-sales\nmode: strict_chunked_transaction_order\nready: false\nstatus: healthy\nalerts: 0\nrepair_plan_required: true\n\nattachment_commands:\n- trellara repair-plan --config trellara.yml\n- trellara quarantine list --config trellara.yml\n\nrecommended_actions:\n- repair target\n",
    )
    .expect("write diagnostics with generic repair action");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let failure_drill = summary
        .gates
        .iter()
        .find(|gate| gate.code == "failure_drill")
        .expect("failure drill gate");
    assert_eq!(
        failure_drill.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(failure_drill
        .missing_markers
        .contains(&"repair_plan_required".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check failure drill generic repair temp dir");
}

#[test]
fn pilot_evidence_check_rejects_required_failure_drill_without_repair_steps() {
    let root = temp_root("pilot-evidence-check-failure-drill-empty-repair");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("diagnostics.txt"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "mode": "strict_chunked_transaction_order",
            "config": "trellara.yml",
            "ready": false,
            "status": "warning",
            "report": {
                "no_target_quarantine": true,
                "recommended_actions": ["run trellara repair-plan before accepting the drill"],
                "proof_checks": [
                    {"code": "target_quarantine", "status": "verified", "evidence": "target quarantine checked"}
                ]
            },
            "repair_plan": {
                "source_id": "local-source",
                "dataset_id": "retail-sales",
                "dry_run": true,
                "plan_required": true,
                "step_count": 0,
                "steps": []
            },
            "attachment_commands": [
                "trellara quarantine list --config trellara.yml"
            ]
        }"#,
    )
    .expect("write diagnostics");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let failure_drill = summary
        .gates
        .iter()
        .find(|gate| gate.code == "failure_drill")
        .expect("failure drill gate");
    assert_eq!(
        failure_drill.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(failure_drill
        .missing_markers
        .contains(&"repair_plan_required".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check failure drill temp dir");
}
