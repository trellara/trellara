use super::*;

#[test]
fn pilot_evidence_check_rejects_verified_apply_without_source_dataset_identity() {
    let root = temp_root("pilot-evidence-check-verified-apply-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales target_relation=public.sales relation_match=true\n",
    )
    .expect("write verified apply without source dataset identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        verified_apply.missing_markers,
        vec!["source dataset identity".to_string()]
    );

    fs::remove_dir_all(root).expect("remove evidence check verified apply identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_verified_apply_without_target_relation_identity() {
    let root = temp_root("pilot-evidence-check-verified-apply-target-identity");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        "source_id=local-source dataset_id=retail-sales converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1 relation=public.sales\n",
    )
    .expect("write verified apply without target relation identity");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);
    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");

    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert_eq!(
        verified_apply.missing_markers,
        vec!["target relation identity".to_string()]
    );

    fs::remove_dir_all(root)
        .expect("remove evidence check verified apply target identity temp dir");
}

#[test]
fn pilot_evidence_check_rejects_verified_apply_without_watermark_context() {
    let root = temp_root("pilot-evidence-check-verified-apply");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        "converged=true\nchecksum_status=match\n",
    )
    .expect("write verified apply without watermark context");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(verified_apply
        .missing_markers
        .contains(&"converged true".to_string()));
    assert!(verified_apply
        .missing_markers
        .contains(&"checksum match".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check verified apply temp dir");
}

#[test]
fn pilot_evidence_check_rejects_verified_apply_with_mismatched_watermarks() {
    let root = temp_root("pilot-evidence-check-verified-apply-watermarks");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B8000 table_count=1\n",
    )
    .expect("write verified apply with mismatched watermarks");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    assert_ne!(summary.verdict, "live_evidence_accepted");
    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(verified_apply
        .missing_markers
        .contains(&"converged true".to_string()));
    assert!(verified_apply
        .missing_markers
        .contains(&"checksum match".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check verified apply watermarks temp dir");
}

#[test]
fn pilot_evidence_check_rejects_verified_apply_with_mismatched_table_count() {
    let root = temp_root("pilot-evidence-check-verified-apply-table-count");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        r#"{
            "source_watermark_lsn": "0/16B6C50",
            "target_watermark_lsn": "0/16B6C50",
            "converged": true,
            "checksum_status": "match",
            "table_count": 2,
            "tables": [
                {"relation": "public.sales", "converged": true, "checksum_status": "match"}
            ]
        }"#,
    )
    .expect("write verified apply with mismatched table count");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(verified_apply
        .missing_markers
        .contains(&"converged true".to_string()));
    assert!(verified_apply
        .missing_markers
        .contains(&"checksum match".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check verified apply table count temp dir");
}

#[test]
fn pilot_evidence_check_rejects_verified_apply_without_table_relation() {
    let root = temp_root("pilot-evidence-check-verified-apply-table-relation");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        "converged=true checksum_status=match source_watermark_lsn=0/16B6C50 target_watermark_lsn=0/16B6C50 table_count=1\n",
    )
    .expect("write verified apply without table relation");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(verified_apply
        .missing_markers
        .contains(&"converged true".to_string()));
    assert!(verified_apply
        .missing_markers
        .contains(&"checksum match".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check verified apply table relation temp dir");
}

#[test]
fn pilot_evidence_check_rejects_verified_apply_with_relation_mismatch() {
    let root = temp_root("pilot-evidence-check-verified-apply-relation-mismatch");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_local_config(&root);
    fs::write(
        evidence_dir.join("verified-apply.txt"),
        r#"{
            "source_id": "local-source",
            "dataset_id": "retail-sales",
            "source_watermark_lsn": "0/16B6C50",
            "target_watermark_lsn": "0/16B6C50",
            "converged": true,
            "checksum_status": "match",
            "table_count": 1,
            "tables": [
                {
                    "relation": "public.sales",
                    "target_relation": "archive.sales",
                    "relation_match": false,
                    "converged": true,
                    "checksum_status": "match"
                }
            ]
        }"#,
    )
    .expect("write verified apply with relation mismatch");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let verified_apply = summary
        .gates
        .iter()
        .find(|gate| gate.code == "verified_apply")
        .expect("verified apply gate");
    assert_eq!(
        verified_apply.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(verified_apply
        .missing_markers
        .contains(&"checksum match".to_string()));

    fs::remove_dir_all(root).expect("remove evidence check verified apply relation temp dir");
}
