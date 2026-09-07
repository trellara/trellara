use super::*;

const VALID_REBALANCE_PLAN: &str = r#"{
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
}"#;

#[test]
fn live_evidence_catalog_maps_partition_rebalance_plan_artifact_and_markers() {
    assert_eq!(
        live_evidence_artifact_name("partition_rebalance_plan"),
        "partition-rebalance-plan.json"
    );
    assert_eq!(
        live_evidence_expected_markers("partition_rebalance_plan"),
        vec![
            "source dataset identity".to_string(),
            "rebalance evidence complete".to_string(),
            "runtime movement disabled".to_string(),
            "visibility contract present".to_string(),
            "skew metrics present".to_string(),
            "move candidates reviewable".to_string(),
        ]
    );

    for marker in live_evidence_expected_markers("partition_rebalance_plan") {
        assert!(
            live_evidence_marker_present("partition_rebalance_plan", &marker, VALID_REBALANCE_PLAN),
            "expected marker {marker} in rebalance plan"
        );
    }
}

#[test]
fn partition_rebalance_skew_metric_marker_accepts_zero_event_plans() {
    let plan = VALID_REBALANCE_PLAN
        .replace(r#""status": "Skewed""#, r#""status": "Stable""#)
        .replace(r#""min_event_count": 100"#, r#""min_event_count": 0"#)
        .replace(r#""max_event_count": 1000"#, r#""max_event_count": 0"#)
        .replace(r#""total_event_count": 1100"#, r#""total_event_count": 0"#)
        .replace(
            r#""skew_ratio_basis_points": 100000"#,
            r#""skew_ratio_basis_points": null"#,
        )
        .replace(
            r#""recommended_moves": [
        {
            "from_partition_id": 0,
            "to_partition_id": 1,
            "estimated_event_delta": 450,
            "reason": "partition 0 has 1000 events while partition 1 has 100 events"
        }
    ]"#,
            r#""recommended_moves": []"#,
        );

    assert!(live_evidence_marker_present(
        "partition_rebalance_plan",
        "skew metrics present",
        &plan
    ));
}

#[test]
fn partition_rebalance_plan_rejects_unsafe_or_incomplete_evidence() {
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "runtime movement disabled",
        &VALID_REBALANCE_PLAN.replace(
            r#""runtime_movement_allowed": false"#,
            r#""runtime_movement_allowed": true"#
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "rebalance evidence complete",
        &VALID_REBALANCE_PLAN.replace(
            r#""evidence_complete": true"#,
            r#""evidence_complete": false"#
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "move candidates reviewable",
        &VALID_REBALANCE_PLAN.replace(r#""estimated_event_delta": 450,"#, "")
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "move candidates reviewable",
        &VALID_REBALANCE_PLAN.replace(r#""to_partition_id": 1"#, r#""to_partition_id": 0"#)
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "move candidates reviewable",
        &VALID_REBALANCE_PLAN.replace(r#""to_partition_id": 1"#, r#""to_partition_id": 2"#)
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "move candidates reviewable",
        &VALID_REBALANCE_PLAN.replace(
            r#""recommended_moves": [
        {
            "from_partition_id": 0,
            "to_partition_id": 1,
            "estimated_event_delta": 450,
            "reason": "partition 0 has 1000 events while partition 1 has 100 events"
        }
    ]"#,
            r#""recommended_moves": []"#
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "skew metrics present",
        &VALID_REBALANCE_PLAN.replace(r#""skew_ratio_basis_points": 100000,"#, "")
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "skew metrics present",
        &VALID_REBALANCE_PLAN.replace(r#""max_skew_percent": 50,"#, "")
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "skew metrics present",
        &VALID_REBALANCE_PLAN.replace(r#""min_event_count": 100"#, r#""min_event_count": 1001"#)
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "skew metrics present",
        &VALID_REBALANCE_PLAN.replace(
            r#""total_event_count": 1100"#,
            r#""total_event_count": 999"#
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "skew metrics present",
        &VALID_REBALANCE_PLAN
            .replace(r#""status": "Skewed""#, r#""status": "Stable""#)
            .replace(r#""max_event_count": 1000"#, r#""max_event_count": 0"#)
            .replace(
                r#""skew_ratio_basis_points": 100000"#,
                r#""skew_ratio_basis_points": null"#
            )
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "visibility contract present",
        &VALID_REBALANCE_PLAN.replace(
            "rebalance is a plan artifact only; runtime ownership movement waits for reviewed manifest, checkpoint, and consumer cutover evidence",
            "rebalance is a plan artifact only; runtime ownership movement waits for consumer cutover evidence"
        )
    ));
    assert!(!live_evidence_marker_present(
        "partition_rebalance_plan",
        "visibility contract present",
        &VALID_REBALANCE_PLAN.replace(
            "rebalance is a plan artifact only; runtime ownership movement waits for reviewed manifest, checkpoint, and consumer cutover evidence",
            "rebalance is a plan artifact only; runtime ownership movement waits for reviewed manifest, checkpoint, and consumer cutover evidence unless operator override is set"
        )
    ));
}
