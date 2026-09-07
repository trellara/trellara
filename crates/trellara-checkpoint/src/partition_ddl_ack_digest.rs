use sha2::{Digest, Sha256};

use crate::PartitionWatermarkSummary;

const ABSENT_VALUE: &str = "-";

pub fn partition_watermark_summary_sha256(watermarks: &PartitionWatermarkSummary) -> String {
    let mut hasher = Sha256::new();
    hash_line(&mut hasher, "source_id", &watermarks.source_id);
    hash_line(&mut hasher, "dataset_id", &watermarks.dataset_id);
    hash_line(
        &mut hasher,
        "expected_partition_count",
        &watermarks.expected_partition_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "observed_partition_count",
        &watermarks.observed_partition_count.to_string(),
    );
    hash_line(
        &mut hasher,
        "complete_partition_set",
        bool_token(watermarks.complete_partition_set),
    );
    hash_line(
        &mut hasher,
        "global_durable_lsn",
        watermarks
            .global_durable_lsn
            .as_deref()
            .unwrap_or(ABSENT_VALUE),
    );
    hash_line(
        &mut hasher,
        "global_applied_lsn",
        watermarks
            .global_applied_lsn
            .as_deref()
            .unwrap_or(ABSENT_VALUE),
    );
    hash_line(
        &mut hasher,
        "global_lag_bytes",
        &watermarks
            .global_durable_to_applied_bytes
            .map(|lag| lag.to_string())
            .unwrap_or_else(|| ABSENT_VALUE.to_string()),
    );
    for missing in &watermarks.missing_partitions {
        hash_line(&mut hasher, "missing_partition", &missing.to_string());
    }
    let mut partitions = watermarks.partitions.iter().collect::<Vec<_>>();
    partitions.sort_by_key(|partition| partition.partition_id);
    for partition in partitions {
        hash_line(
            &mut hasher,
            "partition",
            &format!(
                "{}:{}:{}:{}:{}",
                partition.partition_id,
                partition.last_durable_lsn,
                partition.last_applied_lsn,
                partition.durable_to_applied_bytes,
                bool_token(partition.blocks_global_applied_watermark)
            ),
        );
    }
    format!("{:x}", hasher.finalize())
}

fn hash_line(hasher: &mut Sha256, key: &str, value: &str) {
    hasher.update(key.as_bytes());
    hasher.update(b"=");
    hasher.update(value.as_bytes());
    hasher.update(b"\n");
}

fn bool_token(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}
