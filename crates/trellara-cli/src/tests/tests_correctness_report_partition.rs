use super::*;

#[test]
fn correctness_report_is_not_ready_with_incomplete_partition_watermarks() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(incomplete_partition_watermarks()),
        ..clean_status_parts()
    }));

    assert!(!report.ready);
    assert_eq!(report.proof_check_count, 11);
    assert_eq!(report.at_risk_proof_check_count, 2);
    assert_eq!(report.missing_evidence_proof_check_count, 0);
    assert!(!report.partition_watermark_ready);
    assert_eq!(
        report.transaction_boundary.status,
        TransactionBoundaryStatus::AtRisk
    );
    assert!(report.transaction_boundary.manifest_barrier_required);
    assert_eq!(
        report.transaction_boundary.manifest_barrier_complete,
        Some(false)
    );
    assert_eq!(
        report
            .transaction_boundary
            .global_partition_watermark_caught_up,
        Some(false)
    );
    assert!(report
        .transaction_boundary
        .evidence
        .contains("partition_watermark observed_partitions=2/4"));
    assert!(report
        .transaction_boundary
        .evidence
        .contains("caught_up=false"));
    assert_eq!(report.latest_partition_global_applied_lsn, None);
    assert_eq!(report.issues.len(), 1);
    assert!(report.issues[0].contains("partition watermarks missing"));
    let partition_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "partition_manifest_barrier")
        .expect("partition proof check");
    assert_eq!(partition_check.status, CorrectnessProofStatus::AtRisk);
    assert!(partition_check
        .evidence
        .contains("missing validated manifest checksum/event-count evidence"));
    assert_eq!(
            report.recommended_actions,
            vec![
                "run the barrier-aware applier for all partition topics until partition watermarks are complete and caught up"
            ]
        );
}

#[test]
fn correctness_report_marks_partition_boundary_pending_without_watermarks() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: None,
        ..clean_status_parts()
    }));

    assert_eq!(
        report.transaction_boundary.status,
        TransactionBoundaryStatus::PendingEvidence
    );
    assert_eq!(report.proof_check_count, 11);
    assert_eq!(report.missing_evidence_proof_check_count, 2);
    assert!(report.transaction_boundary.manifest_barrier_required);
    assert_eq!(report.transaction_boundary.manifest_barrier_complete, None);
    assert!(report
        .transaction_boundary
        .evidence
        .contains("partition_watermark missing"));
    assert_eq!(
        report
            .transaction_boundary
            .global_partition_watermark_caught_up,
        None
    );
    assert!(!report.ready);
}

#[test]
fn correctness_report_verified_partition_boundary_mentions_manifest_evidence_headers() {
    let report = CorrectnessReportSummary::from_status(FlowStatusSummary::new(FlowStatusParts {
        mode: "partitioned_scale_mode".to_string(),
        partition_watermarks: Some(complete_partition_watermarks()),
        ..clean_status_parts()
    }));

    assert!(report.ready);
    assert_eq!(report.proof_check_count, 11);
    assert_eq!(report.verified_proof_check_count, 11);
    assert_eq!(
        report.transaction_boundary.status,
        TransactionBoundaryStatus::Verified
    );
    let partition_check = report
        .proof_checks
        .iter()
        .find(|check| check.code == "partition_manifest_barrier")
        .expect("partition proof check");

    assert_eq!(partition_check.status, CorrectnessProofStatus::Verified);
    assert!(partition_check
        .evidence
        .contains("manifest checksum/event-count evidence headers are validated"));
}

fn complete_partition_watermarks() -> PartitionWatermarkSummary {
    PartitionWatermarkSummary {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        expected_partition_count: 2,
        observed_partition_count: 2,
        complete_partition_set: true,
        global_durable_lsn: Some("0/16B6C50".to_string()),
        global_applied_lsn: Some("0/16B6C50".to_string()),
        global_durable_to_applied_bytes: Some(0),
        missing_partitions: Vec::new(),
        partitions: vec![
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 0,
                last_durable_lsn: "0/16B6C50".to_string(),
                last_applied_lsn: "0/16B6C50".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
            trellara_checkpoint::PartitionWatermarkLag {
                partition_id: 1,
                last_durable_lsn: "0/16B6C50".to_string(),
                last_applied_lsn: "0/16B6C50".to_string(),
                durable_to_applied_bytes: 0,
                blocks_global_applied_watermark: false,
            },
        ],
    }
}
