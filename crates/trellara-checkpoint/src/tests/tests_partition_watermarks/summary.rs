use super::*;

#[test]
fn partition_watermark_reports_global_low_watermark() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        3,
        vec![
            partition_checkpoint(0, "0/16B8000", "0/16B7800"),
            partition_checkpoint(1, "0/16B7600", "0/16B7400"),
            partition_checkpoint(2, "0/16B7900", "0/16B7900"),
        ],
    )
    .expect("summary");

    assert!(summary.complete_partition_set);
    assert_eq!(summary.observed_partition_count, 3);
    assert_eq!(summary.global_durable_lsn, Some("0/16B7600".to_string()));
    assert_eq!(summary.global_applied_lsn, Some("0/16B7400".to_string()));
    assert_eq!(summary.global_durable_to_applied_bytes, Some(512));
    assert_eq!(summary.missing_partitions, Vec::<u32>::new());
    assert_eq!(
        summary
            .partitions
            .iter()
            .filter(|partition| partition.blocks_global_applied_watermark)
            .map(|partition| partition.partition_id)
            .collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn partition_watermark_withholds_global_low_watermark_until_all_partitions_report() {
    let summary = PartitionWatermarkSummary::from_checkpoints(
        "source-a",
        "sales",
        4,
        vec![
            partition_checkpoint(0, "0/16B8000", "0/16B7800"),
            partition_checkpoint(2, "0/16B7900", "0/16B7900"),
        ],
    )
    .expect("summary");

    assert!(!summary.complete_partition_set);
    assert_eq!(summary.observed_partition_count, 2);
    assert_eq!(summary.global_durable_lsn, None);
    assert_eq!(summary.global_applied_lsn, None);
    assert_eq!(summary.missing_partitions, vec![1, 3]);
}
