use super::*;

#[test]
fn cli_parses_native_worker_report_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "native",
        "worker-report",
        "--status",
        "source-feedback-ready",
        "--drained-frames",
        "3",
        "--source-feedback-lsn",
        "0/16B6C50",
        "--reason",
        "durable_relay_proof_allows_source_feedback",
        "--format",
        "text",
    ])
    .expect("cli parse");

    assert!(matches!(
        cli.command,
        Command::Native {
            command: NativeCommand::WorkerReport(NativeWorkerReportArgs {
                status: NativeWorkerReportStatus::SourceFeedbackReady,
                drained_frames: 3,
                source_feedback_lsn: Some(lsn),
                reason,
                format: QuickstartOutputFormat::Text
            })
        } if lsn == "0/16B6C50"
            && reason == "durable_relay_proof_allows_source_feedback"
    ));
}
