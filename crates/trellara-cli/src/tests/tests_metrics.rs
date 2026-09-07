use super::*;

#[test]
fn metrics_render_clean_flow_for_scraping() {
    let metrics = render_prometheus_metrics(clean_status());

    assert!(metrics.contains(
            "trellara_flow_ready{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_quickstart_estimated_minutes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 8"
        ));
    assert!(metrics.contains(
            "trellara_quickstart_time_budget_minutes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 10"
        ));
    assert!(metrics.contains(
            "trellara_flow_health_status{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\",status=\"healthy\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_checksum_status{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\",status=\"match\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_validation_current{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_validation_source_lag_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_validation_target_lag_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_status{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\",status=\"verified\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_source_checkpoint_durable{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_target_checkpoint_caught_up{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_manifest_barrier_required{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(!metrics.contains(
            "trellara_transaction_boundary_manifest_barrier_complete{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"}"
        ));
    assert!(metrics.contains(
            "trellara_proof_checks_by_status_total{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\",status=\"verified\"} 10"
        ));
    assert!(metrics.contains(
            "trellara_target_durable_to_applied_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_source_to_target_lag_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_source_stream_spill_threshold_changes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 1024"
        ));
    assert!(metrics.contains(
            "trellara_source_stream_spill_dir_configured{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
}

#[test]
fn metrics_render_source_to_target_watermark_lag() {
    let metrics = render_prometheus_metrics(FlowStatusSummary::new(FlowStatusParts {
        source: Some(CheckpointLag {
            last_seen_lsn: "0/16B8000".to_string(),
            last_durable_lsn: "0/16B8000".to_string(),
            source_is_durable: true,
            ..caught_up_lag()
        }),
        target: Some(caught_up_lag()),
        ..clean_status_parts()
    }));

    assert!(metrics.contains(
            "trellara_transaction_boundary_target_checkpoint_caught_up{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_source_to_target_lag_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 5040"
        ));
}

#[test]
fn metrics_render_partition_and_proof_risk() {
    let metrics = render_prometheus_metrics(FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(incomplete_partition_watermarks()),
        latest_validation: Some(latest_validation(false)),
        ..clean_status_parts()
    }));

    assert!(metrics.contains(
            "trellara_flow_ready{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_quickstart_time_budget_minutes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\"} 10"
        ));
    assert!(metrics.contains(
            "trellara_flow_health_status{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\",status=\"degraded\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_checksum_status{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\",status=\"mismatch\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_manifest_barrier_required{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_manifest_barrier_complete{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_transaction_boundary_global_partition_watermark_caught_up{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_partition_watermark_missing_partitions_total{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\"} 2"
        ));
    assert!(metrics.contains(
            "trellara_proof_checks_by_status_total{source_id=\"source-a\",dataset_id=\"sales\",mode=\"partitioned_scale_mode\",status=\"at_risk\"} 3"
        ));
}

#[test]
fn metrics_render_target_quarantine_reason() {
    let metrics = render_prometheus_metrics(FlowStatusSummary::new(FlowStatusParts {
        latest_quarantine: Some(ApplyQuarantine {
            reason: "no_rows_matched".to_string(),
            ..latest_quarantine()
        }),
        ..clean_status_parts()
    }));

    assert!(metrics.contains(
            "trellara_target_quarantine_blocked{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\",reason=\"no_rows_matched\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_flow_health_status{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\",status=\"blocked\"} 1"
        ));
}

#[test]
fn metrics_render_stale_validation_lag() {
    let mut validation = latest_validation(true);
    validation.source_watermark_lsn = "0/16B6000".to_string();
    validation.target_watermark_lsn = "0/16B6000".to_string();
    let metrics = render_prometheus_metrics(FlowStatusSummary::new(FlowStatusParts {
        latest_validation: Some(validation),
        ..clean_status_parts()
    }));

    assert!(metrics.contains(
            "trellara_flow_ready{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_flow_alerts_total{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 1"
        ));
    assert!(metrics.contains(
            "trellara_validation_current{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 0"
        ));
    assert!(metrics.contains(
            "trellara_validation_source_lag_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 3152"
        ));
    assert!(metrics.contains(
            "trellara_validation_target_lag_bytes{source_id=\"source-a\",dataset_id=\"sales\",mode=\"strict_transaction_order\"} 3152"
        ));
}

#[test]
fn metric_label_values_are_escaped() {
    let mut output = String::new();
    push_metric(
        &mut output,
        "trellara_test_metric",
        &[("source_id", "source\"a"), ("dataset_id", "sales\\north")],
        1,
    );

    assert_eq!(
        output,
        "trellara_test_metric{source_id=\"source\\\"a\",dataset_id=\"sales\\\\north\"} 1\n"
    );
}
