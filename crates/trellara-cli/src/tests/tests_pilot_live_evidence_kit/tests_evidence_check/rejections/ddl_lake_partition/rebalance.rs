use super::*;

#[test]
fn pilot_evidence_check_rejects_rebalance_plan_with_runtime_movement_enabled() {
    let root = temp_root("pilot-evidence-check-rebalance-runtime-movement");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-rebalance-plan.json"),
        valid_rebalance_plan().replace(
            r#""runtime_movement_allowed": false"#,
            r#""runtime_movement_allowed": true"#,
        ),
    )
    .expect("write unsafe rebalance plan");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let rebalance = rebalance_gate(&summary);
    assert_eq!(
        rebalance.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(rebalance
        .missing_markers
        .contains(&"runtime movement disabled".to_string()));

    fs::remove_dir_all(root).expect("remove rebalance runtime movement temp dir");
}

#[test]
fn pilot_evidence_check_rejects_rebalance_plan_without_reviewable_moves() {
    let root = temp_root("pilot-evidence-check-rebalance-move-review");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-rebalance-plan.json"),
        valid_rebalance_plan().replace(r#""estimated_event_delta": 450,"#, ""),
    )
    .expect("write unreviewable rebalance plan");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let rebalance = rebalance_gate(&summary);
    assert_eq!(
        rebalance.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(rebalance
        .missing_markers
        .contains(&"move candidates reviewable".to_string()));

    fs::remove_dir_all(root).expect("remove rebalance move review temp dir");
}

#[test]
fn pilot_evidence_check_rejects_rebalance_plan_with_incomplete_evidence() {
    let root = temp_root("pilot-evidence-check-rebalance-incomplete-evidence");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-rebalance-plan.json"),
        valid_rebalance_plan().replace(
            r#""evidence_complete": true"#,
            r#""evidence_complete": false"#,
        ),
    )
    .expect("write incomplete rebalance plan");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let rebalance = rebalance_gate(&summary);
    assert_eq!(
        rebalance.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(rebalance
        .missing_markers
        .contains(&"rebalance evidence complete".to_string()));

    fs::remove_dir_all(root).expect("remove rebalance incomplete evidence temp dir");
}

#[test]
fn pilot_evidence_check_rejects_rebalance_plan_without_skew_metrics() {
    let root = temp_root("pilot-evidence-check-rebalance-skew-metrics");
    let evidence_dir = root.join("evidence");
    fs::create_dir_all(&evidence_dir).expect("create evidence dir");
    let config_path = write_partitioned_config(&root);
    fs::write(
        evidence_dir.join("partition-rebalance-plan.json"),
        valid_rebalance_plan().replace(r#""skew_ratio_basis_points": 100000,"#, ""),
    )
    .expect("write rebalance plan without skew ratio");
    let config = TrellaraConfig::from_path(&config_path).expect("parse");

    let summary = PilotLiveEvidenceCheckSummary::from_config(&config, &config_path, &evidence_dir);

    let rebalance = rebalance_gate(&summary);
    assert_eq!(
        rebalance.evidence_status,
        PilotLiveEvidenceStatus::Insufficient
    );
    assert!(rebalance
        .missing_markers
        .contains(&"skew metrics present".to_string()));

    fs::remove_dir_all(root).expect("remove rebalance skew metrics temp dir");
}

fn rebalance_gate(summary: &PilotLiveEvidenceCheckSummary) -> &PilotLiveEvidenceGate {
    summary
        .gates
        .iter()
        .find(|gate| gate.code == "partition_rebalance_plan")
        .expect("partition rebalance gate")
}

fn valid_rebalance_plan() -> &'static str {
    r#"{
        "source_id": "local-source",
        "dataset_id": "retail-east",
        "expected_partition_count": 2,
        "policy": "PlanIfSkewed",
        "status": "Skewed",
        "evidence_complete": true,
        "runtime_movement_allowed": false,
        "visibility_contract": "rebalance is a plan artifact only; runtime ownership movement waits for reviewed manifest, checkpoint, and consumer cutover evidence",
        "max_skew_percent": 50,
        "min_event_count": 100,
        "max_event_count": 1000,
        "total_event_count": 1100,
        "skew_ratio_basis_points": 100000,
        "missing_partitions": [],
        "blocking_partition_ids": [],
        "recommended_moves": [
            {
                "from_partition_id": 0,
                "to_partition_id": 1,
                "estimated_event_delta": 450,
                "reason": "partition 0 has 1000 events while partition 1 has 100 events"
            }
        ]
    }"#
}
