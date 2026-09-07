use std::collections::BTreeSet;

use trellara_protocol::parse_lsn;

use crate::LakeEpochSummary;

pub(crate) fn validate_partition_rollup_rows(epoch: &LakeEpochSummary) -> Result<(), String> {
    let mut seen_partitions = BTreeSet::new();
    for partition in &epoch.partition_rollups {
        validate_partition_source_id(&partition.source_id)?;
        if !seen_partitions.insert((partition.source_id.clone(), partition.partition_id)) {
            return Err(format!(
                "partition rollup has duplicate source_id {} partition_id {}",
                partition.source_id, partition.partition_id
            ));
        }
        validate_partition_lsn_window(
            &partition.source_id,
            partition.partition_id,
            partition.first_commit_lsn.as_deref(),
            partition.last_commit_lsn.as_deref(),
        )?;
    }
    Ok(())
}

fn validate_partition_source_id(source_id: &str) -> Result<(), String> {
    if source_id.trim().is_empty() || source_id.trim() != source_id {
        Err(format!(
            "partition rollup has invalid source_id {source_id:?}"
        ))
    } else {
        Ok(())
    }
}

fn validate_partition_lsn_window(
    source_id: &str,
    partition_id: u32,
    first_commit_lsn: Option<&str>,
    last_commit_lsn: Option<&str>,
) -> Result<(), String> {
    let first = parse_required_partition_lsn(
        source_id,
        partition_id,
        "first_commit_lsn",
        first_commit_lsn,
    )?;
    let last =
        parse_required_partition_lsn(source_id, partition_id, "last_commit_lsn", last_commit_lsn)?;
    if first <= last {
        Ok(())
    } else {
        Err(format!(
            "partition rollup source_id {source_id} partition_id {partition_id} has first_commit_lsn after last_commit_lsn"
        ))
    }
}

fn parse_required_partition_lsn(
    source_id: &str,
    partition_id: u32,
    field: &'static str,
    lsn: Option<&str>,
) -> Result<u64, String> {
    let Some(lsn) = lsn else {
        return Err(format!(
            "partition rollup source_id {source_id} partition_id {partition_id} requires {field} evidence"
        ));
    };
    let value = parse_lsn(lsn).map_err(|_| {
        format!(
            "partition rollup source_id {source_id} partition_id {partition_id} has invalid {field} {lsn}"
        )
    })?;
    if value == 0 {
        return Err(format!(
            "partition rollup source_id {source_id} partition_id {partition_id} has invalid {field} {lsn}"
        ));
    }
    Ok(value)
}
