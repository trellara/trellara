use super::*;
use trellara_relay::{NativeWorkerRunReport, NativeWorkerRunStatus};

#[test]
fn native_worker_report_text_explains_source_feedback_boundary() {
    let report = NativeWorkerRunReport {
        status: NativeWorkerRunStatus::SourceFeedbackReady,
        drained_frames: 3,
        source_feedback_lsn: Some("0/16B6C50".to_string()),
        reason: "durable_relay_proof_allows_source_feedback",
    };

    let output =
        render_native_worker_report(&report, QuickstartOutputFormat::Text).expect("render report");

    assert!(output.contains("native worker: source_feedback_ready"));
    assert!(output.contains("drained frames: 3"));
    assert!(output.contains("source feedback lsn: 0/16B6C50"));
    assert!(output.contains("advance source feedback after durable proof"));
}

#[test]
fn native_worker_report_json_is_scriptable() {
    let report = NativeWorkerRunReport {
        status: NativeWorkerRunStatus::FailedClosed,
        drained_frames: 0,
        source_feedback_lsn: None,
        reason: "relay_proof_not_durable",
    };

    let output =
        render_native_worker_report(&report, QuickstartOutputFormat::Json).expect("render report");
    let json: serde_json::Value = serde_json::from_str(&output).expect("json report");

    assert_eq!(json["status"], "failed_closed");
    assert_eq!(json["drained_frames"], 0);
    assert_eq!(json["source_feedback_lsn"], serde_json::Value::Null);
    assert_eq!(
        json["operator_action"],
        "hold source feedback and inspect rejection reason"
    );
}

#[tokio::test]
async fn native_worker_report_command_renders_operator_view() {
    let cli = Cli::try_parse_from([
        "trellara",
        "native",
        "worker-report",
        "--status",
        "slept",
        "--reason",
        "queue_empty_after_supervision_ready",
        "--format",
        "json",
    ])
    .expect("cli parse");

    let output = execute(cli).await.expect("execute native report");

    assert!(output.contains("\"status\": \"slept\""));
    assert!(output.contains("\"operator_action\": \"leave source feedback unchanged\""));
}
