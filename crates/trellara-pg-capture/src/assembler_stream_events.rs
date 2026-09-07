use trellara_protocol::TransactionEnvelope;

use crate::assembler_boundary::{
    canonical_commit_boundary, validate_commit_timestamp_boundary, validate_stream_boundary_id,
};
use crate::assembler_config::TransactionAssemblerConfig;
use crate::{Result, TransactionAssembler};

impl TransactionAssembler {
    pub(crate) fn apply_stream_start(
        &mut self,
        transaction_id: String,
        first_segment: bool,
    ) -> Result<Option<TransactionEnvelope>> {
        validate_stream_boundary_id(&transaction_id, "stream.transaction_id")?;
        self.streamed
            .start(transaction_id, first_segment, &self.stream_spill)?;
        Ok(None)
    }

    pub(crate) fn apply_stream_stop(&mut self) -> Result<Option<TransactionEnvelope>> {
        self.streamed.stop()?;
        Ok(None)
    }

    pub(crate) fn apply_stream_commit(
        &mut self,
        config: &TransactionAssemblerConfig,
        transaction_id: String,
        commit_lsn: String,
        commit_timestamp_ms: i64,
    ) -> Result<Option<TransactionEnvelope>> {
        validate_stream_boundary_id(&transaction_id, "stream.transaction_id")?;
        let commit_lsn =
            canonical_commit_boundary(self.streamed.transaction(&transaction_id)?, &commit_lsn)?;
        validate_commit_timestamp_boundary(&transaction_id, commit_timestamp_ms)?;
        let open = self.streamed.commit(&transaction_id)?;
        self.envelope_from_open(config, open, commit_lsn, commit_timestamp_ms)
    }

    pub(crate) fn apply_stream_abort(
        &mut self,
        transaction_id: String,
        subtransaction_id: String,
    ) -> Result<Option<TransactionEnvelope>> {
        validate_stream_boundary_id(&transaction_id, "stream.transaction_id")?;
        validate_stream_boundary_id(&subtransaction_id, "stream.subtransaction_id")?;
        self.streamed.abort(&transaction_id, &subtransaction_id)?;
        Ok(None)
    }
}
