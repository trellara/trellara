use super::*;

fn input(
    policy: PartitionRebalancePolicy,
    observations: Vec<PartitionLoadObservation>,
) -> PartitionRebalancePlanInput {
    PartitionRebalancePlanInput {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 3,
        policy,
        max_skew_percent: 50,
        observations,
    }
}

fn observation(
    partition_id: u32,
    event_count: u64,
    applied_lsn: &str,
    blocks_global_applied_watermark: bool,
) -> PartitionLoadObservation {
    PartitionLoadObservation {
        partition_id,
        event_count,
        durable_lsn: "0/16B9000".to_string(),
        applied_lsn: applied_lsn.to_string(),
        blocks_global_applied_watermark,
    }
}

#[test]
fn partition_rebalance_plan_reports_stable_complete_evidence() {
    let plan = plan_partition_rebalance(input(
        PartitionRebalancePolicy::PlanIfSkewed,
        vec![
            observation(0, 100, "0/16B9000", false),
            observation(1, 120, "0/16B9000", false),
            observation(2, 130, "0/16B9000", false),
        ],
    ))
    .expect("rebalance plan");

    assert_eq!(plan.status, PartitionRebalanceStatus::Stable);
    assert!(plan.evidence_complete);
    assert!(!plan.runtime_movement_allowed);
    assert_eq!(
        plan.visibility_contract,
        PARTITION_REBALANCE_VISIBILITY_CONTRACT
    );
    assert_eq!(plan.min_event_count, Some(100));
    assert_eq!(plan.max_event_count, Some(130));
    assert_eq!(plan.total_event_count, 350);
    assert_eq!(plan.skew_ratio_basis_points, Some(13_000));
    assert!(plan.recommended_moves.is_empty());
}

#[test]
fn partition_rebalance_plan_recommends_move_only_when_policy_allows() {
    let plan = plan_partition_rebalance(input(
        PartitionRebalancePolicy::PlanIfSkewed,
        vec![
            observation(0, 1_000, "0/16B9000", false),
            observation(1, 100, "0/16B9000", false),
            observation(2, 110, "0/16B9000", false),
        ],
    ))
    .expect("rebalance plan");

    assert_eq!(plan.status, PartitionRebalanceStatus::Skewed);
    assert_eq!(plan.recommended_moves.len(), 1);
    assert_eq!(plan.recommended_moves[0].from_partition_id, 0);
    assert_eq!(plan.recommended_moves[0].to_partition_id, 1);
    assert_eq!(plan.recommended_moves[0].estimated_event_delta, 450);
    assert_eq!(plan.total_event_count, 1_210);
    assert_eq!(plan.skew_ratio_basis_points, Some(100_000));

    let observe_only = plan_partition_rebalance(input(
        PartitionRebalancePolicy::ObserveOnly,
        vec![
            observation(0, 1_000, "0/16B9000", false),
            observation(1, 100, "0/16B9000", false),
            observation(2, 110, "0/16B9000", false),
        ],
    ))
    .expect("observe-only plan");

    assert_eq!(observe_only.status, PartitionRebalanceStatus::Skewed);
    assert!(observe_only.recommended_moves.is_empty());
}

#[test]
fn partition_rebalance_plan_withholds_moves_for_incomplete_or_blocking_evidence() {
    let missing = plan_partition_rebalance(input(
        PartitionRebalancePolicy::PlanIfSkewed,
        vec![
            observation(0, 1_000, "0/16B9000", false),
            observation(2, 100, "0/16B9000", false),
        ],
    ))
    .expect("missing partition plan");

    assert_eq!(missing.status, PartitionRebalanceStatus::IncompleteEvidence);
    assert!(!missing.evidence_complete);
    assert_eq!(missing.missing_partitions, vec![1]);
    assert!(missing.recommended_moves.is_empty());

    let blocking = plan_partition_rebalance(input(
        PartitionRebalancePolicy::PlanIfSkewed,
        vec![
            observation(0, 1_000, "0/16B9000", false),
            observation(1, 100, "0/16B8000", true),
            observation(2, 110, "0/16B9000", false),
        ],
    ))
    .expect("blocking partition plan");

    assert_eq!(
        blocking.status,
        PartitionRebalanceStatus::IncompleteEvidence
    );
    assert_eq!(blocking.blocking_partition_ids, vec![1]);
    assert!(blocking.recommended_moves.is_empty());
}

#[test]
fn partition_rebalance_plan_rejects_invalid_observations() {
    let duplicate = plan_partition_rebalance(input(
        PartitionRebalancePolicy::PlanIfSkewed,
        vec![
            observation(0, 100, "0/16B9000", false),
            observation(0, 110, "0/16B9000", false),
        ],
    ))
    .expect_err("duplicate partition rejected");

    assert!(duplicate.to_string().contains("duplicate observation"));

    let out_of_range = plan_partition_rebalance(input(
        PartitionRebalancePolicy::PlanIfSkewed,
        vec![observation(3, 100, "0/16B9000", false)],
    ))
    .expect_err("out of range partition rejected");

    assert!(out_of_range.to_string().contains("outside expected range"));
}
