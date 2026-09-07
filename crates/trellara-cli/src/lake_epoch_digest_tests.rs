use super::*;
use crate::{LakeEpochPartitionRollup, LakeEpochPartitionSkew};

#[test]
fn manifest_digest_changes_when_table_rollup_changes() {
    let sources = vec![source("store-a", "complete", Some("0/16B6C60"))];
    let mut tables = vec![LakeEpochTableRollup {
        relation: "public.sales".to_string(),
        transaction_count: 1,
        change_count: 1,
        checksum_rollup: 42,
    }];
    let input = LakeEpochManifestDigestInput {
        epoch_id: "epoch-1",
        dataset_id: "retail",
        state: trellara_lake::LakeCompletenessState::Complete,
        straggler_policy: "wait_all_required",
        required_source_count: 1,
        complete_source_count: 1,
        missing_source_count: 0,
        quarantined_source_count: 0,
        transaction_count: 1,
        change_count: 1,
    };
    let partition_skew = empty_partition_skew();
    let before = lake_epoch_manifest_digest(&input, &sources, &tables, &[], &partition_skew, &[]);

    tables[0].change_count = 2;
    let after = lake_epoch_manifest_digest(&input, &sources, &tables, &[], &partition_skew, &[]);

    assert_eq!(before.len(), 64);
    assert_ne!(before, after);
}

#[test]
fn manifest_digest_is_stable_when_rows_are_reordered() {
    let sources = vec![
        source("store-b", "complete", Some("0/16B6C60")),
        source("store-a", "complete", Some("0/16B6C50")),
    ];
    let reversed_sources = sources.iter().cloned().rev().collect::<Vec<_>>();
    let tables = vec![
        LakeEpochTableRollup {
            relation: "public.sales".to_string(),
            transaction_count: 1,
            change_count: 1,
            checksum_rollup: 42,
        },
        LakeEpochTableRollup {
            relation: "public.refunds".to_string(),
            transaction_count: 1,
            change_count: 1,
            checksum_rollup: 43,
        },
    ];
    let reversed_tables = tables.iter().cloned().rev().collect::<Vec<_>>();
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

    assert_eq!(
        lake_epoch_manifest_digest(&input, &sources, &tables, &[], &empty_partition_skew(), &[]),
        lake_epoch_manifest_digest(
            &input,
            &reversed_sources,
            &reversed_tables,
            &[],
            &empty_partition_skew(),
            &[]
        )
    );
}

#[test]
fn manifest_digest_changes_when_partition_rollup_changes() {
    let sources = vec![source("store-a", "complete", Some("0/16B6C60"))];
    let tables = vec![LakeEpochTableRollup {
        relation: "public.sales".to_string(),
        transaction_count: 1,
        change_count: 1,
        checksum_rollup: 42,
    }];
    let mut partitions = vec![LakeEpochPartitionRollup {
        source_id: "store-a".to_string(),
        partition_id: 0,
        first_commit_lsn: Some("0/16B6C60".to_string()),
        last_commit_lsn: Some("0/16B6C60".to_string()),
        transaction_count: 1,
        event_count: 1,
        checksum_rollup: 42,
    }];
    let input = LakeEpochManifestDigestInput {
        epoch_id: "epoch-1",
        dataset_id: "retail",
        state: trellara_lake::LakeCompletenessState::Complete,
        straggler_policy: "wait_all_required",
        required_source_count: 1,
        complete_source_count: 1,
        missing_source_count: 0,
        quarantined_source_count: 0,
        transaction_count: 1,
        change_count: 1,
    };

    let mut partition_skew = LakeEpochPartitionSkew {
        participating_partition_count: 1,
        total_event_count: 1,
        min_event_count: 1,
        max_event_count: 1,
        skew_ratio_basis_points: Some(10_000),
        hottest_partition_ids: vec![0],
        coolest_partition_ids: vec![0],
    };
    let before =
        lake_epoch_manifest_digest(&input, &sources, &tables, &partitions, &partition_skew, &[]);
    partitions[0].event_count = 2;
    partition_skew.total_event_count = 2;
    partition_skew.min_event_count = 2;
    partition_skew.max_event_count = 2;
    let after =
        lake_epoch_manifest_digest(&input, &sources, &tables, &partitions, &partition_skew, &[]);

    assert_eq!(before.len(), 64);
    assert_ne!(before, after);
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

fn empty_partition_skew() -> LakeEpochPartitionSkew {
    LakeEpochPartitionSkew {
        participating_partition_count: 0,
        total_event_count: 0,
        min_event_count: 0,
        max_event_count: 0,
        skew_ratio_basis_points: None,
        hottest_partition_ids: Vec::new(),
        coolest_partition_ids: Vec::new(),
    }
}
