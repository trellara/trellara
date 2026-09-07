use std::collections::{BTreeMap, BTreeSet};

use trellara_protocol::TransactionEnvelope;

use crate::LakeError;

#[derive(Default)]
pub(crate) struct EpochReplayLedger {
    seen_idempotency: BTreeMap<String, u64>,
    seen_transactions: BTreeSet<(String, String, String)>,
}

impl EpochReplayLedger {
    pub(crate) fn validate_idempotency(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        for change in &envelope.changes {
            if let Some(previous_checksum) = self
                .seen_idempotency
                .insert(change.idempotency_key.clone(), envelope.checksum)
            {
                if previous_checksum != envelope.checksum {
                    return Err(LakeError::ConflictingDuplicate {
                        idempotency_key: change.idempotency_key.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    pub(crate) fn is_replayed_transaction(&mut self, envelope: &TransactionEnvelope) -> bool {
        let transaction_key = (
            envelope.source_id.clone(),
            envelope.transaction_id.clone(),
            envelope.commit_lsn.clone(),
        );
        !self.seen_transactions.insert(transaction_key)
    }
}
