use std::collections::BTreeMap;

use crate::raw_cdc_types::{
    LakeRawCdcCommitterTopology, LakeRawCdcDataFilePlan, LakeRawCdcTableCommitter,
    RAW_CDC_SOURCE_ACK_BOUNDARY,
};

pub(super) fn committer_topology(
    data_files: &[LakeRawCdcDataFilePlan],
    source_bucket_count: u32,
) -> LakeRawCdcCommitterTopology {
    let table_committers = table_committers(data_files);
    LakeRawCdcCommitterTopology {
        strategy: "single_table_committer_epoch_batched_append_only_raw_cdc".to_string(),
        committer_count: table_committers.len(),
        table_count: table_committers.len(),
        source_bucket_count: source_bucket_count as usize,
        table_committers,
        source_ack_boundary: RAW_CDC_SOURCE_ACK_BOUNDARY.to_string(),
        catalog_backpressure_rule:
            "slow Iceberg catalog commits backpressure lake fan-in consumers while source WAL remains protected by the durable stream boundary"
                .to_string(),
    }
}

fn table_committers(data_files: &[LakeRawCdcDataFilePlan]) -> Vec<LakeRawCdcTableCommitter> {
    let mut by_table = BTreeMap::<(String, String), TableCommitterCounts>::new();
    for file in data_files {
        let counts = by_table
            .entry((file.table_name.clone(), file.relation.clone()))
            .or_default();
        counts.data_file_count += 1;
        counts.source_buckets.insert(file.source_bucket);
    }

    by_table
        .into_iter()
        .map(|((table_name, relation), counts)| LakeRawCdcTableCommitter {
            table_name,
            relation,
            source_bucket_count: counts.source_buckets.len(),
            data_file_count: counts.data_file_count,
            commit_policy:
                "one committer owns this Iceberg table for the epoch and appends raw CDC files before metadata visibility"
                    .to_string(),
        })
        .collect()
}

#[derive(Default)]
struct TableCommitterCounts {
    data_file_count: usize,
    source_buckets: std::collections::BTreeSet<u32>,
}
