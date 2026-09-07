use trellara_protocol::Checkpoint;

use crate::lsn::parse_lsn;
use crate::{CheckpointError, Result};

pub(crate) fn reject_checkpoint_rewind(existing: &Checkpoint, incoming: &Checkpoint) -> Result<()> {
    reject_lsn_rewind(existing, incoming, "last_seen_lsn")?;
    reject_lsn_rewind(existing, incoming, "last_durable_lsn")?;
    reject_lsn_rewind(existing, incoming, "last_applied_lsn")
}

fn reject_lsn_rewind(
    existing: &Checkpoint,
    incoming: &Checkpoint,
    field: &'static str,
) -> Result<()> {
    let existing_lsn = checkpoint_lsn(existing, field);
    let incoming_lsn = checkpoint_lsn(incoming, field);
    if incoming_lsn.is_empty()
        || existing_lsn.is_empty()
        || parse_lsn(incoming_lsn) >= parse_lsn(existing_lsn)
    {
        return Ok(());
    }

    Err(CheckpointError::Store(format!(
        "checkpoint rewind refused for source_id {:?} dataset_id {:?}: incoming {field} {incoming_lsn:?} is behind existing {field} {existing_lsn:?}",
        existing.source_id, existing.dataset_id
    )))
}

fn checkpoint_lsn<'a>(checkpoint: &'a Checkpoint, field: &str) -> &'a str {
    match field {
        "last_seen_lsn" => &checkpoint.last_seen_lsn,
        "last_durable_lsn" => &checkpoint.last_durable_lsn,
        "last_applied_lsn" => &checkpoint.last_applied_lsn,
        _ => "",
    }
}
