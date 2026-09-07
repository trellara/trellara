use super::*;

#[test]
fn pilot_evidence_check_rejects_contract_preflight_without_check_context() {
    let root = temp_root("pilot-evidence-check-contract-thin");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("contract-test.json"),
        r#"{"source_id":"local-source","dataset_id":"retail-sales","passed":true}"#,
    )
    .expect("write thin contract");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let contract = summary
        .gates
        .iter()
        .find(|gate| gate.code == "contract_preflight")
        .expect("contract preflight gate");
    assert_eq!(
        contract.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(contract
        .missing_markers
        .contains(&"passed true".to_string()));
    assert!(contract
        .missing_markers
        .contains(&"checks present".to_string()));
    assert!(contract
        .missing_markers
        .contains(&"check counts consistent".to_string()));
    assert!(contract
        .missing_markers
        .contains(&"no failed or error checks".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check contract temp dir");
}

#[test]
fn pilot_evidence_check_rejects_contract_preflight_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-contract-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("contract-test.json"),
        r#"{
            "passed": true,
            "check_count": 1,
            "issue_count": 0,
            "recovery_action_count": 0,
            "checks": [
                {
                    "name": "source_and_target_contract:public.sales",
                    "passed": true,
                    "severity": "info",
                    "message": "public.sales satisfies source capture and target compatibility checks",
                    "recommendation": ""
                }
            ],
            "recovery_actions": []
        }"#,
    )
    .expect("write contract without identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let contract = summary
        .gates
        .iter()
        .find(|gate| gate.code == "contract_preflight")
        .expect("contract preflight gate");
    assert_eq!(
        contract.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        contract.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check identity contract temp dir");
}

#[test]
fn pilot_evidence_check_rejects_contract_preflight_with_failed_checks() {
    let root = temp_root("pilot-evidence-check-contract-failed");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("contract-test.json"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "passed": false,
            "check_count": 1,
            "issue_count": 1,
            "recovery_action_count": 1,
            "checks": [
                {
                    "name": "source_and_target_contract:public.sales",
                    "passed": false,
                    "severity": "error",
                    "message": "public.sales has incompatible target schema",
                    "recommendation": "repair schema before running CDC"
                }
            ],
            "recovery_actions": [
                {"code": "repair_schema", "severity": "error", "relations": ["public.sales"], "reason": "incompatible target schema", "command_templates": ["trellara contract-test --config <config>"], "hint": "repair schema"}
            ]
        }"#,
    )
    .expect("write failed contract");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let contract = summary
        .gates
        .iter()
        .find(|gate| gate.code == "contract_preflight")
        .expect("contract preflight gate");
    assert_eq!(
        contract.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(contract
        .missing_markers
        .contains(&"passed true".to_string()));
    assert!(contract
        .missing_markers
        .contains(&"no failed or error checks".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check failed contract temp dir");
}
