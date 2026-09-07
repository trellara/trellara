use crate::LakeEpochPartitionSkew;

use super::lake_epoch_digest_manifest::push_manifest_line;

pub(super) fn push_partition_skew_lines(
    manifest: &mut String,
    partition_skew: &LakeEpochPartitionSkew,
) {
    push_manifest_line(
        manifest,
        "partition_skew",
        &format!(
            "{}|{}|{}|{}|{}|{}|{}",
            partition_skew.participating_partition_count,
            partition_skew.total_event_count,
            partition_skew.min_event_count,
            partition_skew.max_event_count,
            partition_skew
                .skew_ratio_basis_points
                .map(|ratio| ratio.to_string())
                .unwrap_or_default(),
            partition_ids(&partition_skew.hottest_partition_ids),
            partition_ids(&partition_skew.coolest_partition_ids)
        ),
    );
}

fn partition_ids(partition_ids: &[u32]) -> String {
    partition_ids
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
