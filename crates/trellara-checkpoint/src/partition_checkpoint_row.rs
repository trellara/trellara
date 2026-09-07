use crate::{
    validation::validate_partition_checkpoint, CheckpointError, PartitionCheckpoint, Result,
};

pub(crate) fn partition_checkpoint_from_parts(
    source_id: String,
    dataset_id: String,
    partition_id: i32,
    last_durable_lsn: String,
    last_applied_lsn: String,
) -> Result<PartitionCheckpoint> {
    let checkpoint = PartitionCheckpoint {
        source_id,
        dataset_id,
        partition_id: postgres_partition_id(partition_id)?,
        last_durable_lsn,
        last_applied_lsn,
    };
    validate_partition_checkpoint(&checkpoint)?;
    Ok(checkpoint)
}

pub(crate) fn partition_checkpoint_from_row(
    row: tokio_postgres::Row,
) -> Result<PartitionCheckpoint> {
    partition_checkpoint_from_parts(row.get(0), row.get(1), row.get(2), row.get(3), row.get(4))
}

fn postgres_partition_id(partition_id: i32) -> Result<u32> {
    u32::try_from(partition_id).map_err(|_| {
        CheckpointError::Store(format!(
            "postgres returned negative partition id {partition_id}"
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn postgres_partition_id_rejects_negative_values() {
        let error = postgres_partition_id(-1).expect_err("negative partition id");

        assert!(error
            .to_string()
            .contains("postgres returned negative partition id -1"));
    }

    #[test]
    fn postgres_partition_id_accepts_non_negative_values() {
        assert_eq!(postgres_partition_id(42).expect("partition id"), 42);
    }
}
