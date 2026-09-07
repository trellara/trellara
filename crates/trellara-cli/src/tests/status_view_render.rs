use super::*;

mod status_view_partitioned_replay;

#[test]
fn status_view_renders_existing_operator_payloads() {
    let config_path = Path::new("examples/retail-fleet/strict.yml");
    let report = render_status_view(
        clean_status(),
        StatusView::Report,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("report view");
    let alerts = render_status_view(
        clean_status(),
        StatusView::Alerts,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("alerts view");
    let dashboard = render_status_view(
        clean_status(),
        StatusView::Dashboard,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("dashboard view");
    let metrics = render_status_view(
        clean_status(),
        StatusView::Metrics,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("metrics view");
    let diagnostics = render_status_view(
        clean_status(),
        StatusView::Diagnostics,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("diagnostics view");

    assert!(report.contains("\"proof_checks\""));
    assert!(report.contains("\"transaction_boundary\""));
    assert!(report.contains("\"visibility_contract\""));
    assert!(report.contains("\"parallel_replay_contract\""));
    assert!(alerts.contains("\"alert_count\": 0"));
    assert!(dashboard.contains("\"ready\": true"));
    assert!(dashboard.contains("\"source_to_target_lag_bytes\": 0"));
    assert!(dashboard.contains("\"quickstart_time_budget_minutes\": 10"));
    assert!(dashboard.contains("\"proof_check_count\": 10"));
    assert!(metrics.contains("trellara_flow_ready"));
    assert!(diagnostics.contains("\"repair_plan\""));
    assert!(diagnostics.contains("\"barrier_blockers\""));
    assert!(diagnostics.contains("examples/retail-fleet/strict.yml"));
}

#[test]
fn status_view_renders_human_readable_operator_payloads() {
    let config_path = Path::new("examples/retail-fleet/strict.yml");
    let report = render_status_view(
        clean_status(),
        StatusView::Report,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("report view");
    let alerts = render_status_view(
        clean_status(),
        StatusView::Alerts,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("alerts view");
    let dashboard = render_status_view(
        clean_status(),
        StatusView::Dashboard,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("dashboard view");
    let diagnostics = render_status_view(
        clean_status(),
        StatusView::Diagnostics,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("diagnostics view");

    assert!(report.contains("Trellara correctness report"));
    assert!(report.contains("transaction_boundary_evidence:"));
    assert!(report.contains("transaction_visibility_contract:"));
    assert!(report.contains("parallel_replay_contract: parallel replay is disabled"));
    assert!(report.contains("proof_checks:"));
    assert!(alerts.contains("Trellara flow alerts"));
    assert!(alerts.contains("alerts: 0"));
    assert!(dashboard.contains("Trellara dashboard"));
    assert!(dashboard.contains("quickstart: estimated 8 minutes, budget 10 minutes"));
    assert!(dashboard.contains("parallel_replay_contract: parallel replay is disabled"));
    assert!(dashboard.contains("snapshot_handoff: verified"));
    assert!(dashboard.contains("snapshot_handoff_evidence:"));
    assert!(dashboard.contains("- source_to_target_lag_bytes: 0"));
    assert!(diagnostics.contains("Trellara diagnostics"));
    assert!(diagnostics.contains("barrier_blockers:"));
    assert!(diagnostics.contains("attachment_commands:"));
    assert!(diagnostics.contains("examples/retail-fleet/strict.yml"));
}

#[test]
fn report_text_surfaces_issue_codes_for_support_triage() {
    let config_path = Path::new("examples/retail-fleet/strict.yml");
    let status = FlowStatusSummary::new(FlowStatusParts {
        source_slot: lost_wal_slot(),
        latest_validation: None,
        ..clean_status_parts()
    });

    let report = render_status_view(
        status,
        StatusView::Report,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("report view");

    assert!(report.contains("issue_code: source_slot_at_risk"));
    assert!(report.contains("issue_code: checksum_validation_missing_evidence"));
}

#[test]
fn diagnostics_view_renders_manifest_barrier_blockers() {
    let config_path = Path::new("examples/retail-fleet/partitioned.yml");
    let status = FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(incomplete_partition_watermarks()),
        ..clean_status_parts()
    });

    let text = render_status_view(
        status.clone(),
        StatusView::Diagnostics,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("diagnostics text");
    let json = render_status_view(
        status,
        StatusView::Diagnostics,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("diagnostics json");

    assert!(text.contains("- partitioned_scale_mode manifest barrier is not complete"));
    assert!(text.contains("- global partition watermark has not caught up"));
    assert!(json.contains("\"barrier_blockers\""));
    assert!(json.contains("partitioned_scale_mode manifest barrier is not complete"));
}
