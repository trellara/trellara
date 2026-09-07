use super::*;

pub(super) fn assert_fleet_summary_counts(summary: &FleetReportSummary) {
    assert_eq!(summary.flow_count, 2);
    assert_eq!(summary.source_count, 1);
    assert_eq!(summary.dataset_count, 1);
    assert_eq!(summary.table_count, 2);
    assert_eq!(summary.target_configured_count, 2);
    assert_eq!(summary.local_stream_count, 1);
    assert_eq!(summary.kafka_stream_count, 1);
    assert_eq!(summary.partitioned_flow_count, 1);
    assert_eq!(summary.strict_chunked_flow_count, 1);
    assert_eq!(summary.convergence_gate_count, 12);
    assert_eq!(summary.blocked_convergence_gate_count, 0);
    assert_eq!(summary.recovery_drill_count, 16);
    assert_eq!(summary.lake_ready_flow_count, 0);
    assert_eq!(summary.lake_publishable_with_gaps_flow_count, 2);
    assert_eq!(summary.lake_blocked_flow_count, 0);
    assert_eq!(summary.lake_fanin_verdict, "publishable_with_gaps");
    assert_eq!(summary.topology_verdict, "review_required");
}

pub(super) fn assert_fleet_summary_warnings(summary: &FleetReportSummary) {
    assert!(summary.warnings.iter().any(|warning| {
        warning.contains("duplicate flow_id local-source:retail-sales appears 2 times")
    }));
    assert!(summary
        .warnings
        .iter()
        .any(|warning| warning.contains("Kafka-backed flows require broker readiness")));
}
