use super::*;

#[test]
fn pilot_evidence_check_rejects_text_snapshot_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-text-snapshot-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("snapshot.txt"),
        "state=stream_handoff_ready selected_table_count=1 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 watermark_lsn=0/16B6C50",
    )
    .expect("write text snapshot without identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let snapshot = summary
        .gates
        .iter()
        .find(|gate| gate.code == "snapshot_handoff")
        .expect("snapshot handoff gate");
    assert_eq!(
        snapshot.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        snapshot.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check text snapshot identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_snapshot_without_handoff_watermark() {
    let root = temp_root("pilot-evidence-check-snapshot-watermark");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("snapshot.txt"),
        "source_id=local-source dataset_id=retail-sales public.sales copy_complete\nstate=stream_handoff_ready\n",
    )
    .expect("write snapshot without watermark");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let snapshot = summary
        .gates
        .iter()
        .find(|gate| gate.code == "snapshot_handoff")
        .expect("snapshot handoff gate");
    assert_eq!(
        snapshot.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(snapshot
        .missing_markers
        .contains(&"durable handoff watermark".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check temp dir");
}

#[test]
fn pilot_evidence_check_rejects_snapshot_with_mismatched_table_watermark() {
    let root = temp_root("pilot-evidence-check-snapshot-mismatched-watermark");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("snapshot.txt"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "run_id": "pilot-snapshot-1",
            "state": "stream_handoff_ready",
            "slot": "trellara_retail_sales",
            "consistent_lsn": "0/16B6C50",
            "selected_table_count": 1,
            "table_count": 1,
            "tables": [
                {
                    "relation": "public.sales",
                    "state": "copy_complete",
                    "copied_rows": 3,
                    "skipped": false,
                    "watermark_lsn": "0/16B9000"
                }
            ]
        }"#,
    )
    .expect("write mismatched snapshot");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let snapshot = summary
        .gates
        .iter()
        .find(|gate| gate.code == "snapshot_handoff")
        .expect("snapshot handoff gate");
    assert_eq!(
        snapshot.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(snapshot
        .missing_markers
        .contains(&"stream_handoff_ready".to_string()));
    assert!(snapshot
        .missing_markers
        .contains(&"durable handoff watermark".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check mismatched snapshot temp dir");
}

#[test]
fn pilot_evidence_check_rejects_text_snapshot_with_mismatched_table_watermark() {
    let root = temp_root("pilot-evidence-check-text-snapshot-mismatched-watermark");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("snapshot.txt"),
        "source_id=local-source dataset_id=retail-sales state=stream_handoff_ready selected_table_count=1 table_count=1 public.sales copy_complete consistent_lsn=0/16B6C50 watermark_lsn=0/16B9000",
    )
    .expect("write text snapshot with mismatched watermark");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let snapshot = summary
        .gates
        .iter()
        .find(|gate| gate.code == "snapshot_handoff")
        .expect("snapshot handoff gate");
    assert_eq!(
        snapshot.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(snapshot
        .missing_markers
        .contains(&"durable handoff watermark".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check text mismatched snapshot temp dir");
}
