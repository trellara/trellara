use super::*;

#[test]
fn pilot_evidence_check_rejects_snapshot_with_malformed_handoff_lsn() {
    let root = temp_root("pilot-evidence-check-snapshot-malformed-lsn");
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
            "consistent_lsn": "not-a-lsn",
            "selected_table_count": 1,
            "table_count": 1,
            "tables": [
                {
                    "relation": "public.sales",
                    "state": "copy_complete",
                    "copied_rows": 3,
                    "skipped": false,
                    "watermark_lsn": "not-a-lsn"
                }
            ]
        }"#,
    )
    .expect("write malformed snapshot");
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
        .contains(&"copy_complete".to_string()));
    assert!(snapshot
        .missing_markers
        .contains(&"durable handoff watermark".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check malformed snapshot temp dir");
}
