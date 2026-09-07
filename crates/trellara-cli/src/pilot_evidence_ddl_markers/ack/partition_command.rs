use std::collections::BTreeSet;

use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn};

pub(super) fn args_are_valid(
    barrier_lsn: Option<String>,
    expected_count: Option<String>,
    durable_lsns: Vec<String>,
    applied_lsns: Vec<String>,
) -> bool {
    let Some(barrier_lsn) = barrier_lsn.filter(|lsn| lsn_is_valid(lsn)) else {
        return false;
    };
    let Some(expected_count) = expected_count
        .and_then(|count| count.parse::<u32>().ok())
        .filter(|count| *count > 0)
    else {
        return false;
    };

    partition_lsn_args_cover_expected_count(&durable_lsns, expected_count, &barrier_lsn)
        && partition_lsn_args_cover_expected_count(&applied_lsns, expected_count, &barrier_lsn)
}

fn partition_lsn_args_cover_expected_count(
    args: &[String],
    expected_count: u32,
    barrier_lsn: &str,
) -> bool {
    if args.len() != expected_count as usize {
        return false;
    }

    let mut partitions = BTreeSet::new();
    args.iter().all(|arg| {
        partition_lsn_arg(arg, expected_count, barrier_lsn)
            .is_some_and(|partition_id| partitions.insert(partition_id))
    }) && partitions.len() == expected_count as usize
}

fn partition_lsn_arg(arg: &str, expected_count: u32, barrier_lsn: &str) -> Option<u32> {
    let (partition_id, lsn) = arg.split_once('=')?;
    let partition_id = partition_id.parse::<u32>().ok()?;
    (partition_id < expected_count && lsn_is_valid(lsn) && parse_lsn(lsn) >= parse_lsn(barrier_lsn))
        .then_some(partition_id)
}

fn lsn_is_valid(lsn: &str) -> bool {
    lsn_shape_is_valid(lsn) && parse_lsn(lsn) > 0
}
