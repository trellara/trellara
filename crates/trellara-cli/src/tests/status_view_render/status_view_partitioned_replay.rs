use super::*;

#[test]
fn status_report_surfaces_partitioned_parallel_replay_contract() {
    let config_path = Path::new("examples/retail-fleet/partitioned.yml");
    let status = FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(incomplete_partition_watermarks()),
        ..clean_status_parts()
    });

    let text = render_status_view(
        status.clone(),
        StatusView::Report,
        QuickstartOutputFormat::Text,
        config_path,
    )
    .expect("report text");
    let json = render_status_view(
        status,
        StatusView::Dashboard,
        QuickstartOutputFormat::Json,
        config_path,
    )
    .expect("dashboard json");

    assert!(text.contains(
        "parallel_replay_contract: partition workers may replay DML-only committed transactions by partition"
    ));
    assert!(text
        .contains("DDL-only and mixed DDL/DML transactions must use the DDL barrier release path"));
    assert!(json.contains("\"parallel_replay_contract\""));
    assert!(json.contains("post-DDL DML becomes globally visible"));
}
