use crate::assembler_spill_config::StreamSpillConfig;
use crate::assembler_transaction::OpenTransaction;
use crate::{CaptureError, Result};
use std::collections::HashMap;

#[derive(Default)]
pub(crate) struct StreamedTransactions {
    active_transaction_id: Option<String>,
    transactions: HashMap<String, OpenTransaction>,
}

impl StreamedTransactions {
    pub(crate) fn start(
        &mut self,
        transaction_id: String,
        first_segment: bool,
        spill_config: &StreamSpillConfig,
    ) -> Result<()> {
        if let Some(active_transaction_id) = &self.active_transaction_id {
            return Err(CaptureError::StreamStartBeforeStop {
                transaction_id,
                active_transaction_id: active_transaction_id.clone(),
            });
        }
        if first_segment {
            if self.transactions.contains_key(&transaction_id) {
                return Err(CaptureError::NestedBegin(transaction_id));
            }
            self.transactions.insert(
                transaction_id.clone(),
                OpenTransaction {
                    transaction_id: transaction_id.clone(),
                    begin_lsn: String::new(),
                    changes: spill_config.pending_change_buffer(),
                    ddl_events: Vec::new(),
                },
            );
        } else if !self.transactions.contains_key(&transaction_id) {
            return Err(CaptureError::ChangeOutsideTransaction);
        }
        self.active_transaction_id = Some(transaction_id);
        Ok(())
    }

    pub(crate) fn stop(&mut self) -> Result<()> {
        if self.active_transaction_id.is_none() {
            return Err(CaptureError::StreamStopWithoutStart);
        }
        self.active_transaction_id = None;
        Ok(())
    }

    pub(crate) fn commit(&mut self, transaction_id: &str) -> Result<OpenTransaction> {
        if let Some(active_transaction_id) = &self.active_transaction_id {
            return Err(CaptureError::StreamCommitBeforeStop {
                transaction_id: transaction_id.to_string(),
                active_transaction_id: active_transaction_id.clone(),
            });
        }
        self.active_transaction_id = None;
        self.transactions
            .remove(transaction_id)
            .ok_or(CaptureError::CommitWithoutBegin)
    }

    pub(crate) fn transaction(&self, transaction_id: &str) -> Result<&OpenTransaction> {
        self.transactions
            .get(transaction_id)
            .ok_or(CaptureError::CommitWithoutBegin)
    }

    pub(crate) fn abort(&mut self, transaction_id: &str, subtransaction_id: &str) -> Result<()> {
        if !self.transactions.contains_key(transaction_id) {
            return Err(CaptureError::ChangeOutsideTransaction);
        }

        if transaction_id == subtransaction_id {
            self.transactions.remove(transaction_id);
            if self.active_transaction_id.as_deref() == Some(transaction_id) {
                self.active_transaction_id = None;
            }
            return Ok(());
        }

        if let Some(open) = self.transactions.get_mut(transaction_id) {
            open.retain_stream_subtransaction_changes(subtransaction_id)?;
        }
        Ok(())
    }

    pub(crate) fn open_for_active_event(
        &mut self,
        event_transaction_id: &str,
    ) -> Result<&mut OpenTransaction> {
        let Some(parent_id) = &self.active_transaction_id else {
            return Err(CaptureError::ChangeOutsideTransaction);
        };
        if parent_id != event_transaction_id && self.transactions.contains_key(event_transaction_id)
        {
            return Err(CaptureError::InvalidTransactionBoundary {
                transaction_id: event_transaction_id.to_string(),
                reason: format!(
                    "stream.event_transaction_id names a different open streamed transaction while active streamed transaction is {parent_id}"
                ),
            });
        }
        self.transactions
            .get_mut(parent_id)
            .ok_or(CaptureError::ChangeOutsideTransaction)
    }

    #[cfg(test)]
    pub(crate) fn get(&self, transaction_id: &str) -> Option<&OpenTransaction> {
        self.transactions.get(transaction_id)
    }

    #[cfg(test)]
    pub(crate) fn contains_key(&self, transaction_id: &str) -> bool {
        self.transactions.contains_key(transaction_id)
    }
}
