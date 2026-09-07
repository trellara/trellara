use crate::assembler_transaction::OpenTransaction;
use crate::lsn::{format_lsn, parse_lsn};
use crate::{CaptureError, Result};

pub(crate) fn canonical_begin_boundary(transaction_id: &str, begin_lsn: &str) -> Result<String> {
    validate_transaction_id(transaction_id)?;
    canonical_nonzero_capture_lsn(transaction_id, "begin_lsn", begin_lsn)
}

pub(crate) fn canonical_commit_boundary(
    open: &OpenTransaction,
    commit_lsn: &str,
) -> Result<String> {
    validate_transaction_id(&open.transaction_id)?;
    let commit_lsn_value =
        parse_nonzero_capture_lsn(&open.transaction_id, "commit_lsn", commit_lsn)?;
    if !open.begin_lsn.trim().is_empty() {
        let begin_lsn_value =
            parse_nonzero_capture_lsn(&open.transaction_id, "begin_lsn", &open.begin_lsn)?;
        if begin_lsn_value > commit_lsn_value {
            return Err(CaptureError::InvalidTransactionBoundary {
                transaction_id: open.transaction_id.clone(),
                reason: format!(
                    "begin_lsn {} is after commit_lsn {commit_lsn}",
                    open.begin_lsn
                ),
            });
        }
    }
    Ok(format_lsn(commit_lsn_value))
}

pub(crate) fn validate_commit_timestamp_boundary(
    transaction_id: &str,
    commit_timestamp_ms: i64,
) -> Result<()> {
    if commit_timestamp_ms > 0 {
        return Ok(());
    }
    Err(CaptureError::InvalidTransactionBoundary {
        transaction_id: transaction_id.to_string(),
        reason: "commit_timestamp_ms must be greater than zero".to_string(),
    })
}

pub(crate) fn validate_stream_boundary_id(transaction_id: &str, field: &str) -> Result<()> {
    if transaction_id.trim().is_empty() {
        return Err(CaptureError::InvalidTransactionBoundary {
            transaction_id: transaction_id.to_string(),
            reason: format!("{field} must not be empty"),
        });
    }
    if transaction_id != transaction_id.trim() {
        return Err(CaptureError::InvalidTransactionBoundary {
            transaction_id: transaction_id.to_string(),
            reason: format!("{field} must not contain surrounding whitespace"),
        });
    }
    Ok(())
}

fn validate_transaction_id(transaction_id: &str) -> Result<()> {
    validate_stream_boundary_id(transaction_id, "transaction_id")
}

fn parse_nonzero_capture_lsn(transaction_id: &str, field: &str, lsn: &str) -> Result<u64> {
    let parsed = parse_lsn(lsn).map_err(|error| CaptureError::InvalidTransactionBoundary {
        transaction_id: transaction_id.to_string(),
        reason: format!("{field} {lsn:?} is invalid: {error}"),
    })?;
    if parsed == 0 {
        return Err(CaptureError::InvalidTransactionBoundary {
            transaction_id: transaction_id.to_string(),
            reason: format!("{field} must be greater than zero"),
        });
    }
    Ok(parsed)
}

fn canonical_nonzero_capture_lsn(transaction_id: &str, field: &str, lsn: &str) -> Result<String> {
    Ok(format_lsn(parse_nonzero_capture_lsn(
        transaction_id,
        field,
        lsn,
    )?))
}
