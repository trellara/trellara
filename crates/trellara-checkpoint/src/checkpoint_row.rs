use trellara_protocol::Checkpoint;

use crate::{checkpoint_validation::validate_checkpoint, Result};

pub(crate) fn checkpoint_from_parts(
    source_id: String,
    dataset_id: String,
    last_seen_lsn: String,
    last_durable_lsn: String,
    last_applied_lsn: String,
) -> Result<Checkpoint> {
    let checkpoint = Checkpoint {
        source_id,
        dataset_id,
        last_seen_lsn,
        last_durable_lsn,
        last_applied_lsn,
    };
    validate_checkpoint(&checkpoint)?;
    Ok(checkpoint)
}

pub(crate) fn checkpoint_from_row(row: tokio_postgres::Row) -> Result<Checkpoint> {
    checkpoint_from_parts(row.get(0), row.get(1), row.get(2), row.get(3), row.get(4))
}
