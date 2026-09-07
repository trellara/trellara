use super::*;

#[test]
fn correctness_report_rejects_target_that_is_only_locally_caught_up() {
    let mut status = correctness_status("strict_transaction_order");
    status.source = Some(CheckpointLag {
        last_seen_lsn: "0/16B8000".to_string(),
        last_durable_lsn: "0/16B8000".to_string(),
        source_is_durable: true,
        ..caught_up_lag()
    });
    status.target = Some(caught_up_lag());

    let report = CorrectnessReportSummary::from_status(status);

    assert!(!report.ready);
    assert!(!report.target_caught_up);
    assert_eq!(
        report.transaction_boundary.status,
        TransactionBoundaryStatus::AtRisk
    );
    assert!(!report.transaction_boundary.target_checkpoint_caught_up);
    assert!(report
        .transaction_boundary
        .evidence
        .contains("source_to_target_progress source_durable_lsn=0/16B8000"));
    assert!(report
        .transaction_boundary
        .evidence
        .contains("target_applied_lsn=0/16B6C50"));
    assert!(report
        .transaction_boundary
        .evidence
        .contains("caught_up=false"));

    let target = report
        .proof_checks
        .iter()
        .find(|check| check.code == "target_checkpoint")
        .expect("target checkpoint proof");
    assert_eq!(target.status, CorrectnessProofStatus::AtRisk);
    assert!(target
        .evidence
        .contains("but behind source durable LSN 0/16B8000"));
    assert!(target
        .recommendation
        .as_deref()
        .expect("recommendation")
        .contains("target applied watermark reaches the source durable watermark"));
    assert!(report
        .recommended_actions
        .iter()
        .any(|action| action
            .contains("target applied watermark reaches the source durable watermark")));
}
