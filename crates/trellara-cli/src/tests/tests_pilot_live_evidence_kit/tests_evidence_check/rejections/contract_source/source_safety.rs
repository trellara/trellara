use super::*;

#[test]
fn pilot_evidence_check_rejects_source_safety_without_findings_context() {
    let root = temp_root("pilot-evidence-check-source-safety-thin");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("source-safety.txt"),
        "Trellara source safety\nsource: local-source\ndataset: retail-sales\nstatus: healthy\ngrade: A (100/100)\n",
    )
    .expect("write source safety without findings context");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let source_safety = summary
        .gates
        .iter()
        .find(|gate| gate.code == "source_safety")
        .expect("source safety gate");
    assert_eq!(
        source_safety.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(source_safety
        .missing_markers
        .contains(&"no critical findings".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check source safety thin temp dir");
}

#[test]
fn pilot_evidence_check_rejects_source_safety_with_generic_findings_context() {
    let root = temp_root("pilot-evidence-check-source-safety-generic-findings");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("source-safety.txt"),
        "Trellara source safety\nsource: local-source\ndataset: retail-sales\nstatus: healthy\ngrade: A (100/100)\n\nfindings:\noperator reviewed source posture\n",
    )
    .expect("write source safety with generic findings context");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let source_safety = summary
        .gates
        .iter()
        .find(|gate| gate.code == "source_safety")
        .expect("source safety gate");
    assert_eq!(
        source_safety.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(source_safety
        .missing_markers
        .contains(&"no critical findings".to_string()));

    fs::remove_dir_all(root)
        .expect("remove evidence check source safety generic findings temp dir");
}

#[test]
fn pilot_evidence_check_rejects_source_safety_with_inconsistent_factor_counts() {
    let root = temp_root("pilot-evidence-check-source-safety-factor-counts");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("source-safety.txt"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "status": "degraded",
            "factor_count": 2,
            "critical_factor_count": 0,
            "factors": [
                {
                    "code": "source_slot_failover_disabled",
                    "severity": "warning",
                    "evidence": "slot failover=false",
                    "recommendation": "enable failover slot before promotion drills"
                }
            ]
        }"#,
    )
    .expect("write source safety with inconsistent factor counts");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let source_safety = summary
        .gates
        .iter()
        .find(|gate| gate.code == "source_safety")
        .expect("source safety gate");
    assert_eq!(
        source_safety.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(source_safety
        .missing_markers
        .contains(&"no critical findings".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check source safety factor counts temp dir");
}

#[test]
fn pilot_evidence_check_distinguishes_missing_and_insufficient_artifacts() {
    let root = temp_root("pilot-evidence-check-missing");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("source-safety.txt"),
        "Trellara source safety\nsource: local-source\ndataset: retail-sales\nstatus: blocked\n- [critical] source_slot_missing\n",
    )
    .expect("write insufficient source safety");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_eq!(summary.verdict, "missing_live_evidence");
    assert!(summary.missing_gate_count > 0);
    assert_eq!(summary.insufficient_gate_count, 1);
    let source_safety = summary
        .gates
        .iter()
        .find(|gate| gate.code == "source_safety")
        .expect("source safety gate");
    assert_eq!(
        source_safety.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(source_safety
        .missing_markers
        .contains(&"no critical findings".to_string()));
    assert!(summary
        .next_commands
        .iter()
        .any(|command| command.contains("trellara check --config")));

    fs::remove_dir_all(root).expect("remove evidence check temp dir");
}
