use tokio_postgres::Client;
use trellara_checkpoint::TransactionKey;
use trellara_protocol::TransactionEnvelope;

use crate::checkpoint_quarantine_sql::RECORD_QUARANTINED_ENVELOPE;
use crate::{ApplyError, Result};

pub(crate) fn quarantine_reason(error: &ApplyError) -> &'static str {
    match error {
        ApplyError::Postgres(_) => "target_postgres_error",
        ApplyError::Protocol(_) => "protocol_error",
        ApplyError::Checkpoint(_) => "checkpoint_error",
        ApplyError::UnsupportedOperation(_) => "unsupported_operation",
        ApplyError::MissingRelation { .. } => "missing_relation",
        ApplyError::MissingSchemaVersionEvidence { .. } => "missing_schema_version_evidence",
        ApplyError::MissingRowImage { .. } => "missing_row_image",
        ApplyError::MissingKeyColumns { .. } => "missing_key_columns",
        ApplyError::UnchangedToastKeyColumn { .. } => "unchanged_toast_key_column",
        ApplyError::NoMutableColumns { .. } => "no_mutable_columns",
        ApplyError::UnsupportedValueKind { .. } => "unsupported_value_kind",
        ApplyError::PlaceholderRangeOverflow { .. } => "placeholder_range_overflow",
        ApplyError::MissingCheckpointEvidence { .. } => "missing_checkpoint_evidence",
        ApplyError::CheckpointBoundaryMismatch { .. } => "checkpoint_boundary_mismatch",
        ApplyError::InvalidCheckpointManifestBoundaryMode { .. } => {
            "invalid_checkpoint_manifest_boundary_mode"
        }
        ApplyError::NoRowsMatched { .. } => "no_rows_matched",
        ApplyError::DdlApplyBlocked { .. } => "ddl_apply_blocked",
        ApplyError::DdlDmlReleaseBlocked { .. } => "ddl_dml_release_blocked",
        ApplyError::DdlBarrierRequired { .. } => "ddl_barrier_required",
        ApplyError::MissingDdlField { .. } => "missing_ddl_field",
        ApplyError::InvalidDdlField { .. } => "invalid_ddl_field",
        ApplyError::DdlSchemaVersionMismatch { .. } => "ddl_schema_version_mismatch",
        ApplyError::InvalidDdlAckEvidence { .. } => "invalid_ddl_ack_evidence",
        ApplyError::UnsafeDdlStatement { .. } => "unsafe_ddl_statement",
    }
}

pub(crate) async fn record_quarantined_envelope(
    client: &Client,
    envelope: &TransactionEnvelope,
    error: &ApplyError,
) -> Result<u64> {
    let record = quarantined_envelope_record(envelope, error)?;
    Ok(client
        .execute(
            RECORD_QUARANTINED_ENVELOPE,
            &[
                &record.transaction_key.source_id,
                &record.transaction_key.database_id,
                &record.transaction_key.dataset_id,
                &record.transaction_key.transaction_id,
                &record.transaction_key.commit_lsn,
                &record.reason,
                &record.detail,
            ],
        )
        .await?)
}

#[derive(Debug)]
pub(crate) struct QuarantinedEnvelopeRecord {
    pub(crate) transaction_key: TransactionKey,
    pub(crate) reason: &'static str,
    pub(crate) detail: String,
}

pub(crate) fn quarantined_envelope_record(
    envelope: &TransactionEnvelope,
    error: &ApplyError,
) -> Result<QuarantinedEnvelopeRecord> {
    let transaction_key = TransactionKey::try_from_envelope(envelope)?;
    Ok(QuarantinedEnvelopeRecord {
        transaction_key,
        reason: quarantine_reason(error),
        detail: error.to_string(),
    })
}
