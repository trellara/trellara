use super::*;
use proptest::prelude::*;

proptest! {
    #[test]
    fn partition_watermark_property_uses_lowest_complete_partition_lsn(
        (expected_partition_count, watermarks) in partition_watermarks_strategy(),
    ) {
        let checkpoints = watermarks
            .iter()
            .enumerate()
            .map(|(partition_id, (durable, applied))| {
                partition_checkpoint(
                    partition_id as u32,
                    &lsn_string(*durable),
                    &lsn_string(*applied),
                )
            })
            .collect::<Vec<_>>();
        let min_durable = watermarks
            .iter()
            .map(|(durable, _)| *durable)
            .min()
            .expect("non-empty watermarks");
        let min_applied = watermarks
            .iter()
            .map(|(_, applied)| *applied)
            .min()
            .expect("non-empty watermarks");

        let summary = PartitionWatermarkSummary::from_checkpoints(
            "source-a",
            "sales",
            expected_partition_count,
            checkpoints,
        )
        .expect("summary");

        prop_assert!(summary.complete_partition_set);
        prop_assert_eq!(summary.observed_partition_count, expected_partition_count);
        prop_assert!(summary.missing_partitions.is_empty());
        prop_assert_eq!(summary.global_durable_lsn, Some(lsn_string(min_durable)));
        prop_assert_eq!(summary.global_applied_lsn, Some(lsn_string(min_applied)));
        prop_assert_eq!(
            summary.global_durable_to_applied_bytes,
            Some(min_durable.saturating_sub(min_applied))
        );
    }

    #[test]
    fn partition_watermark_property_withholds_global_lsn_for_incomplete_sets(
        (expected_partition_count, checkpoints) in incomplete_partition_watermarks_strategy(),
    ) {
        let summary = PartitionWatermarkSummary::from_checkpoints(
            "source-a",
            "sales",
            expected_partition_count,
            checkpoints,
        )
        .expect("summary");

        prop_assert!(!summary.complete_partition_set);
        prop_assert!(summary.observed_partition_count < expected_partition_count);
        prop_assert!(!summary.missing_partitions.is_empty());
        prop_assert_eq!(summary.global_durable_lsn, None);
        prop_assert_eq!(summary.global_applied_lsn, None);
        prop_assert_eq!(summary.global_durable_to_applied_bytes, None);
        prop_assert!(summary
            .partitions
            .iter()
            .all(|partition| !partition.blocks_global_applied_watermark));
    }
}
