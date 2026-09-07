use trellara_checkpoint::{CheckpointStore, FlowKey};
use trellara_protocol::{format_lsn, parse_lsn, Checkpoint, TransactionEnvelope};

use crate::{RelayError, Result};

pub(crate) async fn record_durable<C>(
    checkpoint_store: &C,
    envelope: &TransactionEnvelope,
) -> Result<String>
where
    C: CheckpointStore,
{
    let commit_lsn = canonical_lsn(&envelope.commit_lsn)?;
    let flow = FlowKey::new(&envelope.source_id, &envelope.dataset_id);
    let existing = checkpoint_store.load_checkpoint(&flow).await?;
    if let Some(existing) = existing {
        validate_loaded_checkpoint_order(&existing)?;
        if lsn_gte(&existing.last_durable_lsn, &commit_lsn)? {
            return canonical_lsn(&existing.last_durable_lsn);
        }
        checkpoint_store
            .save_checkpoint(Checkpoint {
                source_id: envelope.source_id.clone(),
                dataset_id: envelope.dataset_id.clone(),
                last_seen_lsn: max_lsn(&existing.last_seen_lsn, &commit_lsn)?,
                last_durable_lsn: commit_lsn.clone(),
                last_applied_lsn: existing.last_applied_lsn,
            })
            .await?;
        return Ok(commit_lsn);
    }

    checkpoint_store
        .save_checkpoint(Checkpoint {
            source_id: envelope.source_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            last_seen_lsn: commit_lsn.clone(),
            last_durable_lsn: commit_lsn.clone(),
            last_applied_lsn: String::new(),
        })
        .await?;
    Ok(commit_lsn)
}

fn canonical_lsn(lsn: &str) -> Result<String> {
    Ok(format_lsn(parse_lsn(lsn)?))
}

fn lsn_gte(left: &str, right: &str) -> Result<bool> {
    Ok(parse_lsn(left)? >= parse_lsn(right)?)
}

fn max_lsn(left: &str, right: &str) -> Result<String> {
    if lsn_gte(left, right)? {
        canonical_lsn(left)
    } else {
        canonical_lsn(right)
    }
}

fn validate_loaded_checkpoint_order(checkpoint: &Checkpoint) -> Result<()> {
    validate_loaded_lsn_not_ahead(
        "last_durable_lsn",
        &checkpoint.last_durable_lsn,
        "last_seen_lsn",
        &checkpoint.last_seen_lsn,
    )?;
    validate_loaded_lsn_not_ahead(
        "last_applied_lsn",
        &checkpoint.last_applied_lsn,
        "last_durable_lsn",
        &checkpoint.last_durable_lsn,
    )
}

fn validate_loaded_lsn_not_ahead(
    field: &'static str,
    value: &str,
    boundary_field: &'static str,
    boundary: &str,
) -> Result<()> {
    if value.trim().is_empty()
        || boundary.trim().is_empty()
        || parse_lsn(value)? <= parse_lsn(boundary)?
    {
        return Ok(());
    }
    Err(RelayError::CheckpointWatermarkInconsistent {
        field,
        value: canonical_lsn(value)?,
        boundary_field,
        boundary: canonical_lsn(boundary)?,
    })
}
