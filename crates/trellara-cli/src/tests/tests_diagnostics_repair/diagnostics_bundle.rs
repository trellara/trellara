use super::*;

#[test]
fn diagnostics_bundle_collects_clean_flow_evidence() {
    let bundle = DiagnosticsBundleSummary::from_status(
        clean_status(),
        Path::new("examples/retail-fleet/strict.yml"),
    );

    assert!(bundle.ready);
    assert_eq!(bundle.status, FlowHealthStatus::Healthy);
    assert_eq!(bundle.latest_failure, None);
    assert_eq!(bundle.report.proof_check_count, 10);
    assert_eq!(bundle.alerts.alert_count, 0);
    assert!(!bundle.repair_plan.plan_required);
    assert!(bundle.barrier_blockers.is_empty());
    assert!(bundle.metrics.contains("trellara_flow_ready"));
    assert_eq!(
        bundle.attachment_commands,
        vec![
            "trellara status --config examples/retail-fleet/strict.yml --view report --format text"
                .to_string(),
            "trellara status --config examples/retail-fleet/strict.yml --view dashboard --format text"
                .to_string(),
            "trellara status --config examples/retail-fleet/strict.yml --view metrics".to_string(),
            "trellara status --config examples/retail-fleet/strict.yml --view diagnostics --format text"
                .to_string(),
            "trellara repair-plan --config examples/retail-fleet/strict.yml".to_string(),
            "trellara quarantine list --config examples/retail-fleet/strict.yml".to_string(),
        ]
    );
}

#[test]
fn diagnostics_bundle_surfaces_manifest_barrier_blockers() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(incomplete_partition_watermarks()),
        ..clean_status_parts()
    });
    let bundle = DiagnosticsBundleSummary::from_status(
        status,
        Path::new("examples/retail-fleet/partitioned.yml"),
    );

    assert!(!bundle.ready);
    assert_eq!(
        bundle.barrier_blockers,
        vec![
            "partitioned_scale_mode manifest barrier is not complete".to_string(),
            "global partition watermark has not caught up".to_string(),
        ]
    );

    let text = render_diagnostics_text(&bundle);
    assert!(text.contains("barrier_blockers:"));
    assert!(text.contains("- partitioned_scale_mode manifest barrier is not complete"));
    assert!(text.contains("- global partition watermark has not caught up"));
}

#[test]
fn diagnostics_bundle_preserves_blocked_recovery_plan() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_quarantine: Some(latest_quarantine()),
        recovery_actions: vec![blocked_recovery_action()],
        ..clean_status_parts()
    });
    let bundle = DiagnosticsBundleSummary::from_status(
        status,
        Path::new("examples/retail-fleet/strict.yml"),
    );

    assert!(!bundle.ready);
    assert_eq!(bundle.status, FlowHealthStatus::Blocked);
    assert_eq!(
        bundle
            .latest_failure
            .as_ref()
            .map(|failure| failure.code.as_str()),
        Some("target_quarantine_blocked")
    );
    assert!(bundle.repair_plan.plan_required);
    assert_eq!(bundle.repair_plan.step_count, 1);
    assert_eq!(bundle.alerts.alert_count, 1);
    assert!(bundle.metrics.contains("trellara_flow_ready"));
}
