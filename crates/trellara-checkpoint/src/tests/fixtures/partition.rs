use super::*;
use proptest::prelude::*;

pub(in crate::tests) fn partition_checkpoint(
    partition_id: u32,
    last_durable_lsn: &str,
    last_applied_lsn: &str,
) -> PartitionCheckpoint {
    PartitionCheckpoint {
        source_id: "source-a".to_string(),
        dataset_id: "sales".to_string(),
        partition_id,
        last_durable_lsn: last_durable_lsn.to_string(),
        last_applied_lsn: last_applied_lsn.to_string(),
    }
}

pub(in crate::tests) fn lsn_string(value: u64) -> String {
    format!("{:X}/{:X}", value >> 32, value & 0xffff_ffff)
}

pub(in crate::tests) fn partition_watermarks_strategy(
) -> impl Strategy<Value = (u32, Vec<(u64, u64)>)> {
    (1u32..16).prop_flat_map(|expected_partition_count| {
        let len = expected_partition_count as usize;
        (
            Just(expected_partition_count),
            prop::collection::vec((1u64..(1u64 << 40), 0u64..1_000_000), len),
        )
            .prop_map(|(count, rows)| {
                let watermarks = rows
                    .into_iter()
                    .map(|(applied, durable_delta)| (applied + durable_delta, applied))
                    .collect();
                (count, watermarks)
            })
    })
}

pub(in crate::tests) fn incomplete_partition_watermarks_strategy(
) -> impl Strategy<Value = (u32, Vec<PartitionCheckpoint>)> {
    (2u32..16).prop_flat_map(|expected_partition_count| {
        let max_observed_count = expected_partition_count as usize;
        (
            Just(expected_partition_count),
            prop::collection::btree_set(0u32..expected_partition_count, 0..max_observed_count),
            prop::collection::vec((1u64..(1u64 << 40), 0u64..1_000_000), 0..max_observed_count),
        )
            .prop_map(|(count, observed_partitions, rows)| {
                let checkpoints = observed_partitions
                    .into_iter()
                    .zip(rows)
                    .map(|(partition_id, (applied, durable_delta))| {
                        partition_checkpoint(
                            partition_id,
                            &lsn_string(applied + durable_delta),
                            &lsn_string(applied),
                        )
                    })
                    .collect();
                (count, checkpoints)
            })
    })
}
