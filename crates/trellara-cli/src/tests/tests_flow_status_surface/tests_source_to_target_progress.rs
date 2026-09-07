use super::*;

#[test]
fn flow_status_reports_source_to_target_watermark_lag_failure() {
    let summary = FlowStatusSummary::new(FlowStatusParts {
        source: Some(source_at_newer_durable_lsn()),
        target: Some(caught_up_lag()),
        ..clean_status_parts()
    });

    let failure = summary.latest_failure.expect("latest failure");

    assert_eq!(failure.code, "target_source_watermark_lag");
    assert_eq!(failure.severity, FlowAlertSeverity::Warning);
    assert!(failure
        .message
        .contains("target applied LSN 0/16B6C50 is behind source durable LSN 0/16B8000"));
    assert!(summary
        .health
        .issues
        .iter()
        .any(|issue| issue.contains("behind source durable LSN 0/16B8000")));
}

fn source_at_newer_durable_lsn() -> CheckpointLag {
    CheckpointLag {
        last_seen_lsn: "0/16B8000".to_string(),
        last_durable_lsn: "0/16B8000".to_string(),
        source_is_durable: true,
        ..caught_up_lag()
    }
}
