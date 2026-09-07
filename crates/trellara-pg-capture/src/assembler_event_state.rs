use trellara_protocol::TransactionEnvelope;

use crate::assembler_boundary::{
    canonical_begin_boundary, canonical_commit_boundary, validate_commit_timestamp_boundary,
    validate_stream_boundary_id,
};
use crate::assembler_buffer::PendingChangeBuffer;
use crate::assembler_config::TransactionAssemblerConfig;
use crate::assembler_schema_versions;
use crate::assembler_transaction::OpenTransaction;
use crate::{CaptureError, Result, TransactionAssembler};

impl TransactionAssembler {
    pub(crate) fn apply_begin(
        &mut self,
        transaction_id: String,
        begin_lsn: String,
    ) -> Result<Option<TransactionEnvelope>> {
        let begin_lsn = canonical_begin_boundary(&transaction_id, &begin_lsn)?;
        if let Some(open) = &self.open {
            return Err(CaptureError::NestedBegin(open.transaction_id.clone()));
        }
        self.open = Some(OpenTransaction {
            transaction_id,
            begin_lsn,
            changes: PendingChangeBuffer::memory(usize::MAX),
            ddl_events: Vec::new(),
        });
        Ok(None)
    }

    pub(crate) fn open_transaction_for_event(
        &mut self,
        transaction_id: Option<&str>,
    ) -> Result<&mut OpenTransaction> {
        if let Some(transaction_id) = transaction_id {
            validate_stream_boundary_id(transaction_id, "stream.event_transaction_id")?;
            self.streamed.open_for_active_event(transaction_id)
        } else {
            self.open
                .as_mut()
                .ok_or(CaptureError::ChangeOutsideTransaction)
        }
    }

    pub(crate) fn envelope_from_open(
        &self,
        config: &TransactionAssemblerConfig,
        open: OpenTransaction,
        commit_lsn: String,
        commit_timestamp_ms: i64,
    ) -> Result<Option<TransactionEnvelope>> {
        let commit_lsn = canonical_commit_boundary(&open, &commit_lsn)?;
        validate_commit_timestamp_boundary(&open.transaction_id, commit_timestamp_ms)?;
        if open.is_empty() {
            return Ok(None);
        }
        let schema_versions = assembler_schema_versions::schema_versions_for_transaction(
            &self.relation_schema_versions,
            open.changed_relation_oids()?,
            &open.ddl_events,
        )?;
        open.into_envelope(config, commit_lsn, commit_timestamp_ms, schema_versions)
            .map(Some)
    }
}
