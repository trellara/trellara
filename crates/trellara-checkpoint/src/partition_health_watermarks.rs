use crate::{parse_lsn, PartitionWatermarkLag, PartitionWatermarkSummary};

pub(crate) fn partition_stragglers(
    summary: &PartitionWatermarkSummary,
    max_observed_applied_lsn: Option<&String>,
) -> Vec<u32> {
    let Some(max_observed_applied_lsn) = max_observed_applied_lsn else {
        return Vec::new();
    };
    let max_observed_applied = parse_lsn(max_observed_applied_lsn);

    summary
        .partitions
        .iter()
        .filter(|partition| parse_lsn(&partition.last_applied_lsn) < max_observed_applied)
        .map(|partition| partition.partition_id)
        .collect()
}

pub(crate) fn global_low_watermark_partition_ids<'a>(
    summary: &'a PartitionWatermarkSummary,
    low_watermark_lsn: Option<&String>,
    partition_lsn: impl Fn(&'a PartitionWatermarkLag) -> &'a String,
) -> Vec<u32> {
    if !summary.complete_partition_set {
        return Vec::new();
    }
    let Some(low_watermark_lsn) = low_watermark_lsn else {
        return Vec::new();
    };
    let low_watermark = parse_lsn(low_watermark_lsn);

    summary
        .partitions
        .iter()
        .filter(|partition| parse_lsn(partition_lsn(partition)) == low_watermark)
        .map(|partition| partition.partition_id)
        .collect()
}

pub(crate) fn max_lsn<'a>(lsns: impl Iterator<Item = &'a String>) -> Option<String> {
    lsns.max_by_key(|lsn| parse_lsn(lsn)).cloned()
}

pub(crate) fn lsn_skew<'a>(lsns: impl Iterator<Item = &'a String>) -> Option<u64> {
    let mut values = lsns.map(|lsn| parse_lsn(lsn));
    let first = values.next()?;
    let (min, max) = values.fold((first, first), |(min, max), value| {
        (min.min(value), max.max(value))
    });
    Some(max.saturating_sub(min))
}
