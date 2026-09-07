use super::*;

#[test]
fn pilot_evidence_check_rejects_source_safety_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-source-safety-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("source-safety.txt"),
        "Trellara source safety\nstatus: healthy\ngrade: A (100/100)\n\nfindings:\n- none\n",
    )
    .expect("write source safety without identity");
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
    assert_eq!(
        source_safety.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check source safety identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_source_safety_for_wrong_dataset_identity() {
    let root = temp_root("pilot-evidence-check-source-safety-wrong-dataset");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("source-safety.txt"),
        "Trellara source safety\nsource: local-source\ndataset: warehouse-sales\nstatus: healthy\ngrade: A (100/100)\n\nfindings:\n- none\n",
    )
    .expect("write source safety with wrong dataset identity");
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
    assert_eq!(
        source_safety.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check wrong source safety identity temp dir");
}
