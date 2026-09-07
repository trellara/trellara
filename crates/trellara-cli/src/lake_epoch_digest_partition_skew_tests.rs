use super::*;
use crate::{LakeEpochPartitionRollup, LakeEpochPartitionSkew, LakeEpochSourceWatermark};

#[test]
fn manifest_digest_changes_when_partition_skew_changes() {
    let sources = vec![source("store-a", "complete", Some("0/16B6C60"))];
    let tables = vec![LakeEpochTableRollup {
        relation: "public.sales".to_string(),
        transaction_count: 1,
        change_count: 2,
        checksum_rollup: 42,
    }];
    let partitions = vec![
        LakeEpochPartitionRollup {
            source_id: "store-a".to_string(),
            partition_id: 0,
            first_commit_lsn: Some("0/16B6C60".to_string()),
            last_commit_lsn: Some("0/16B6C60".to_string()),
            transaction_count: 1,
            event_count: 1,
            checksum_rollup: 42,
        },
        LakeEpochPartitionRollup {
            source_id: "store-b".to_string(),
            partition_id: 1,
            first_commit_lsn: Some("0/16B6C70".to_string()),
            last_commit_lsn: Some("0/16B6C70".to_string()),
            transaction_count: 1,
            event_count: 1,
            checksum_rollup: 43,
        },
    ];
    let input = LakeEpochManifestDigestInput {
        epoch_id: "epoch-1",
        dataset_id: "retail",
        state: trellara_lake::LakeCompletenessState::Complete,
        straggler_policy: "wait_all_required",
        required_source_count: 2,
        complete_source_count: 2,
        missing_source_count: 0,
        quarantined_source_count: 0,
        transaction_count: 2,
        change_count: 2,
    };
    let balanced = LakeEpochPartitionSkew {
        participating_partition_count: 2,
        total_event_count: 2,
        min_event_count: 1,
        max_event_count: 1,
        skew_ratio_basis_points: Some(10_000),
        hottest_partition_ids: vec![0, 1],
        coolest_partition_ids: vec![0, 1],
    };
    let mut stale_skew = balanced.clone();
    stale_skew.max_event_count = 2;

    assert_ne!(
        lake_epoch_manifest_digest(&input, &sources, &tables, &partitions, &balanced, &[]),
        lake_epoch_manifest_digest(&input, &sources, &tables, &partitions, &stale_skew, &[])
    );
}

fn source(id: &str, state: &str, end_lsn: Option<&str>) -> LakeEpochSourceWatermark {
    LakeEpochSourceWatermark {
        source_id: id.to_string(),
        state: state.to_string(),
        start_lsn: end_lsn.map(str::to_string),
        end_lsn: end_lsn.map(str::to_string),
        transaction_count: usize::from(end_lsn.is_some()),
        change_count: usize::from(end_lsn.is_some()),
        gap_reason: None,
    }
}
