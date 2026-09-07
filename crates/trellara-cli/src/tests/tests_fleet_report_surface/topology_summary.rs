use super::*;

mod local_flow;
mod partitioned_flow;
mod summary_counts;

#[test]
fn fleet_report_summarizes_multiple_flow_configs() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-report-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet report temp dir");
    let local_path = root.join("local.yml");
    let partitioned_path = root.join("partitioned.yml");
    let strict_chunked_local_yaml = local_stream_yaml().replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 1000",
    );
    fs::write(&local_path, strict_chunked_local_yaml).expect("write local config");
    fs::write(&partitioned_path, partitioned_yaml()).expect("write partitioned config");

    let summary = FleetReportSummary::from_args(&FleetReportArgs {
        config: vec![local_path.clone(), partitioned_path.clone()],
        format: QuickstartOutputFormat::Json,
    })
    .expect("fleet report");

    summary_counts::assert_fleet_summary_counts(&summary);
    summary_counts::assert_fleet_summary_warnings(&summary);
    local_flow::assert_local_flow(&summary, &local_path);
    partitioned_flow::assert_partitioned_flow(&summary, &partitioned_path);
    partitioned_flow::assert_fleet_lake_proof_commands(&summary);

    fs::remove_dir_all(root).expect("remove fleet report temp dir");
}
