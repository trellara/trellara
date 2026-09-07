use trellara_protocol::TransactionEnvelope;

use crate::assembler_boundary::{canonical_commit_boundary, validate_commit_timestamp_boundary};
use crate::assembler_config::TransactionAssemblerConfig;
use crate::assembler_ddl::PendingDdlEvent;
use crate::assembler_schema_versions;
use crate::{CaptureError, LogicalEvent, Result, TransactionAssembler};

impl TransactionAssembler {
    pub fn apply(
        &mut self,
        config: &TransactionAssemblerConfig,
        event: LogicalEvent,
    ) -> Result<Option<TransactionEnvelope>> {
        match event {
            LogicalEvent::Begin {
                transaction_id,
                begin_lsn,
            } => self.apply_begin(transaction_id, begin_lsn),
            LogicalEvent::Change {
                transaction_id,
                relation,
                operation,
                replica_identity,
                before,
                after,
            } => {
                self.apply_change_event(
                    transaction_id,
                    relation,
                    operation,
                    replica_identity,
                    before,
                    after,
                )?;
                Ok(None)
            }
            LogicalEvent::Truncate {
                transaction_id,
                relations,
            } => {
                self.apply_truncate_event(transaction_id, relations)?;
                Ok(None)
            }
            LogicalEvent::Ddl {
                transaction_id,
                operation,
                relation,
                statement,
                schema_fingerprint_before,
                schema_fingerprint_after,
                target_auto_apply,
                release_gate,
            } => {
                self.apply_ddl_event(
                    transaction_id,
                    PendingDdlEvent {
                        operation,
                        relation,
                        statement,
                        schema_fingerprint_before,
                        schema_fingerprint_after,
                        target_auto_apply,
                        release_gate,
                    },
                )?;
                Ok(None)
            }
            LogicalEvent::RelationMetadata {
                relation,
                schema_fingerprint,
            } => {
                assembler_schema_versions::record_relation_schema_version(
                    &mut self.relation_schema_versions,
                    relation,
                    schema_fingerprint,
                );
                Ok(None)
            }
            LogicalEvent::Commit {
                commit_lsn,
                commit_timestamp_ms,
            } => {
                let Some(open) = self.open.as_ref() else {
                    return Err(CaptureError::CommitWithoutBegin);
                };
                let commit_lsn = canonical_commit_boundary(open, &commit_lsn)?;
                validate_commit_timestamp_boundary(&open.transaction_id, commit_timestamp_ms)?;
                let open = self.open.take().expect("validated open transaction");
                self.envelope_from_open(config, open, commit_lsn, commit_timestamp_ms)
            }
            LogicalEvent::Abort => {
                self.open = None;
                Ok(None)
            }
            LogicalEvent::StreamStart {
                transaction_id,
                first_segment,
            } => self.apply_stream_start(transaction_id, first_segment),
            LogicalEvent::StreamStop => self.apply_stream_stop(),
            LogicalEvent::StreamCommit {
                transaction_id,
                commit_lsn,
                commit_timestamp_ms,
            } => self.apply_stream_commit(config, transaction_id, commit_lsn, commit_timestamp_ms),
            LogicalEvent::StreamAbort {
                transaction_id,
                subtransaction_id,
            } => self.apply_stream_abort(transaction_id, subtransaction_id),
        }
    }
}
