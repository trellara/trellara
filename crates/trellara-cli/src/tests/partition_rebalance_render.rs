use super::*;

fn watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 2,
        observed_partition_count: 2,
        complete_partition_set: true,
        global_durable_lsn: Some("0/16B9000".to_string()),
        global_applied_lsn: Some("0/16B9000".to_string()),
        global_durable_to_applied_bytes: Some(0),
        missing_partitions: Vec::new(),
        partitions: vec![
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 0,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B9000".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 1,
                last_durable_lsn: "0/16B9000".to_string(),
                last_applied_lsn: "0/16B9000".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
        ],
    }
}

#[test]
fn partition_rebalance_plan_text_reports_observe_only_skew_without_moves() {
    let args = PartitionRebalancePlanArgs {
        config: PathBuf::from("partitioned.yml"),
        format: PilotGuideOutputFormat::Text,
        max_skew_percent: 50,
        plan_moves: false,
        partition_event_counts: vec!["0=1000".to_string(), "1=100".to_string()],
    };

    let plan =
        partition_rebalance_plan_from_watermarks(&watermarks(), &args).expect("rebalance plan");
    let output = render_partition_rebalance_plan(&plan, args.format).expect("render");

    assert_eq!(plan.status, PartitionRebalanceStatus::Skewed);
    assert!(plan.recommended_moves.is_empty());
    assert!(output.contains("Trellara partition rebalance plan"));
    assert!(output.contains("status: skewed"));
    assert!(output.contains("runtime_movement_allowed: false"));
    assert!(output.contains("total_event_count: 1100"));
    assert!(output.contains("max_skew_percent: 50"));
    assert!(output.contains("skew_ratio_basis_points: 100000"));
    assert!(output.contains("recommended_moves: 0"));
}

#[test]
fn partition_rebalance_plan_json_includes_reviewable_move_candidates() {
    let args = PartitionRebalancePlanArgs {
        config: PathBuf::from("partitioned.yml"),
        format: PilotGuideOutputFormat::Json,
        max_skew_percent: 50,
        plan_moves: true,
        partition_event_counts: vec!["0=1000".to_string(), "1=100".to_string()],
    };

    let plan =
        partition_rebalance_plan_from_watermarks(&watermarks(), &args).expect("rebalance plan");
    let output = render_partition_rebalance_plan(&plan, args.format).expect("render");

    assert_eq!(plan.recommended_moves[0].from_partition_id, 0);
    assert_eq!(plan.recommended_moves[0].to_partition_id, 1);
    assert!(output.contains("\"status\": \"Skewed\""));
    assert!(output.contains("\"runtime_movement_allowed\": false"));
    assert!(output.contains("\"total_event_count\": 1100"));
    assert!(output.contains("\"skew_ratio_basis_points\": 100000"));
    assert!(output.contains("\"recommended_moves\""));
}

#[test]
fn partition_rebalance_plan_requires_event_counts_for_observed_partitions() {
    let args = PartitionRebalancePlanArgs {
        config: PathBuf::from("partitioned.yml"),
        format: PilotGuideOutputFormat::Json,
        max_skew_percent: 50,
        plan_moves: true,
        partition_event_counts: vec!["0=1000".to_string()],
    };

    let error = partition_rebalance_plan_from_watermarks(&watermarks(), &args)
        .expect_err("missing event count");

    assert!(error
        .to_string()
        .contains("missing --partition-event-count 1=<count>"));
}
