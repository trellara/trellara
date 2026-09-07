use std::collections::BTreeMap;

use crate::{LakeEpochPartitionRollup, LakeEpochPartitionSkew};

pub(crate) fn lake_epoch_partition_skew(
    partitions: &[LakeEpochPartitionRollup],
) -> LakeEpochPartitionSkew {
    let mut events_by_partition = BTreeMap::<u32, usize>::new();
    for partition in partitions {
        *events_by_partition
            .entry(partition.partition_id)
            .or_default() += partition.event_count;
    }

    let total_event_count = events_by_partition.values().sum();
    let min_event_count = events_by_partition
        .values()
        .copied()
        .min()
        .unwrap_or_default();
    let max_event_count = events_by_partition
        .values()
        .copied()
        .max()
        .unwrap_or_default();

    LakeEpochPartitionSkew {
        participating_partition_count: events_by_partition.len(),
        total_event_count,
        min_event_count,
        max_event_count,
        skew_ratio_basis_points: skew_ratio_basis_points(min_event_count, max_event_count),
        hottest_partition_ids: partition_ids_with_count(&events_by_partition, max_event_count),
        coolest_partition_ids: partition_ids_with_count(&events_by_partition, min_event_count),
    }
}

fn skew_ratio_basis_points(min_event_count: usize, max_event_count: usize) -> Option<u64> {
    if min_event_count == 0 {
        return None;
    }
    Some((max_event_count as u64).saturating_mul(10_000) / min_event_count as u64)
}

fn partition_ids_with_count(events_by_partition: &BTreeMap<u32, usize>, count: usize) -> Vec<u32> {
    events_by_partition
        .iter()
        .filter_map(|(partition_id, event_count)| (*event_count == count).then_some(*partition_id))
        .collect()
}
