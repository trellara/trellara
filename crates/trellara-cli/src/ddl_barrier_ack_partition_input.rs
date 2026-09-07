use std::collections::BTreeMap;

use trellara_checkpoint::{lsn_shape_is_valid, parse_lsn, PartitionCheckpoint};

use crate::{CliError, Result, TrellaraConfig};

pub(crate) fn partition_checkpoints(
    config: &TrellaraConfig,
    durable_values: &[String],
    applied_values: &[String],
) -> Result<Vec<PartitionCheckpoint>> {
    let durable = partition_lsn_map(durable_values, "partition-durable-lsn", "durable_lsn")?;
    let applied = partition_lsn_map(applied_values, "partition-applied-lsn", "applied_lsn")?;
    if durable.keys().ne(applied.keys()) {
        return Err(CliError::InvalidConfig(
            "partition durable and applied LSN evidence must cover the same partition ids"
                .to_string(),
        ));
    }

    durable
        .into_iter()
        .map(|(partition_id, durable_lsn)| {
            let applied_lsn = applied
                .get(&partition_id)
                .expect("validated partition id coverage");
            Ok(PartitionCheckpoint {
                source_id: config.source.id.clone(),
                dataset_id: config.dataset.id.clone(),
                partition_id,
                last_durable_lsn: durable_lsn,
                last_applied_lsn: applied_lsn.clone(),
            })
        })
        .collect()
}

fn partition_lsn_map(
    values: &[String],
    flag: &'static str,
    value_label: &'static str,
) -> Result<BTreeMap<u32, String>> {
    if values.is_empty() {
        return Err(CliError::InvalidConfig(format!(
            "schema ddl-barrier ack --sink partition_visibility requires --{flag}"
        )));
    }
    let mut partitions = BTreeMap::new();
    for value in values {
        let (partition_id, lsn) = value.split_once('=').ok_or_else(|| {
            CliError::InvalidConfig(format!(
                "{flag} {value} must use <partition_id>=<{value_label}>"
            ))
        })?;
        if partition_id.trim() != partition_id || lsn.trim() != lsn {
            return Err(CliError::InvalidConfig(format!(
                "{flag} {value} must not contain surrounding whitespace"
            )));
        }
        validate_partition_lsn(flag, value, lsn)?;
        let partition_id = partition_id.parse::<u32>().map_err(|source| {
            CliError::InvalidConfig(format!("{flag} {value} has invalid partition id: {source}"))
        })?;
        if partitions.insert(partition_id, lsn.to_string()).is_some() {
            return Err(CliError::InvalidConfig(format!(
                "{flag} {value} duplicates partition id {partition_id}"
            )));
        }
    }
    Ok(partitions)
}

fn validate_partition_lsn(flag: &str, value: &str, lsn: &str) -> Result<()> {
    if !lsn_shape_is_valid(lsn) || parse_lsn(lsn) == 0 {
        return Err(CliError::InvalidConfig(format!(
            "{flag} {value} must include a non-zero PostgreSQL LSN like 0/16B9000"
        )));
    }
    Ok(())
}
