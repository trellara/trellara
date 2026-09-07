use trellara_protocol::Checkpoint;

use crate::lsn::parse_lsn;
use crate::lsn_validation::validate_optional_nonzero_lsn;
use crate::validation_guards::{require_no_surrounding_whitespace, require_non_empty};
use crate::CheckpointError;
use crate::FlowKey;
use crate::Result;

pub(crate) fn validate_flow_key(flow: &FlowKey) -> Result<()> {
    validate_flow_identity("flow key", "source_id", &flow.source_id)?;
    validate_flow_identity("flow key", "dataset_id", &flow.dataset_id)
}

pub(crate) fn validate_checkpoint(checkpoint: &Checkpoint) -> Result<()> {
    validate_flow_identity("checkpoint", "source_id", &checkpoint.source_id)?;
    validate_flow_identity("checkpoint", "dataset_id", &checkpoint.dataset_id)?;
    validate_optional_nonzero_lsn(
        "checkpoint",
        "last_seen_lsn",
        Some(&checkpoint.last_seen_lsn),
    )?;
    validate_optional_nonzero_lsn(
        "checkpoint",
        "last_durable_lsn",
        Some(&checkpoint.last_durable_lsn),
    )?;
    validate_optional_nonzero_lsn(
        "checkpoint",
        "last_applied_lsn",
        Some(&checkpoint.last_applied_lsn),
    )?;
    validate_ack_order(checkpoint)
}

fn validate_flow_identity(context: &str, field: &'static str, value: &str) -> Result<()> {
    require_non_empty(value, format!("{context} {field} must not be empty"))?;
    require_no_surrounding_whitespace(
        value,
        format!("{context} {field} must not contain surrounding whitespace"),
    )
}

fn validate_ack_order(checkpoint: &Checkpoint) -> Result<()> {
    validate_lsn_not_ahead(
        "last_durable_lsn",
        &checkpoint.last_durable_lsn,
        "last_seen_lsn",
        &checkpoint.last_seen_lsn,
    )?;
    validate_lsn_not_ahead(
        "last_applied_lsn",
        &checkpoint.last_applied_lsn,
        "last_durable_lsn",
        &checkpoint.last_durable_lsn,
    )
}

fn validate_lsn_not_ahead(
    field: &'static str,
    value: &str,
    boundary_field: &'static str,
    boundary: &str,
) -> Result<()> {
    if value.trim().is_empty()
        || boundary.trim().is_empty()
        || parse_lsn(value) <= parse_lsn(boundary)
    {
        return Ok(());
    }
    Err(CheckpointError::Store(format!(
        "checkpoint {field} {value:?} must not be ahead of {boundary_field} {boundary:?}"
    )))
}
