use std::collections::BTreeMap;

use trellara_protocol::TransactionEnvelope;

use super::dedup_key::{transaction_identity, RawCdcIdempotencyKey, RawCdcTransactionKey};
use crate::LakeError;

#[derive(Debug, Eq, PartialEq)]
pub(super) enum RawCdcDedupOutcome {
    New { transaction_id: String },
    Duplicate,
}

#[derive(Default)]
pub(super) struct RawCdcDedupLedger {
    seen_transactions: BTreeMap<RawCdcTransactionKey, u64>,
    seen_idempotency: BTreeMap<RawCdcIdempotencyKey, u64>,
}

impl RawCdcDedupLedger {
    pub(super) fn record(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<RawCdcDedupOutcome, LakeError> {
        let outcome = self.validate_transaction_evidence(envelope)?;
        self.validate_idempotency_evidence(envelope)?;
        self.record_idempotency_evidence(envelope);
        self.record_transaction_evidence(envelope)?;
        Ok(outcome)
    }

    fn validate_idempotency_evidence(
        &self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        for change in &envelope.changes {
            let idempotency_key = RawCdcIdempotencyKey::new(envelope, change);
            if let Some(previous_checksum) = self.seen_idempotency.get(&idempotency_key) {
                if *previous_checksum != envelope.checksum {
                    return Err(LakeError::ConflictingDuplicate {
                        idempotency_key: change.idempotency_key.clone(),
                    });
                }
            }
        }
        Ok(())
    }

    fn validate_transaction_evidence(
        &self,
        envelope: &TransactionEnvelope,
    ) -> Result<RawCdcDedupOutcome, LakeError> {
        let transaction_key = RawCdcTransactionKey::new(envelope)?;
        if let Some(previous_checksum) = self.seen_transactions.get(&transaction_key) {
            if *previous_checksum != envelope.checksum {
                return Err(LakeError::ConflictingDuplicate {
                    idempotency_key: envelope
                        .changes
                        .first()
                        .map(|change| change.idempotency_key.clone())
                        .map_or_else(|| transaction_identity(envelope), Ok)?,
                });
            }
            return Ok(RawCdcDedupOutcome::Duplicate);
        }

        Ok(RawCdcDedupOutcome::New {
            transaction_id: transaction_identity(envelope)?,
        })
    }

    fn record_idempotency_evidence(&mut self, envelope: &TransactionEnvelope) {
        for change in &envelope.changes {
            let idempotency_key = RawCdcIdempotencyKey::new(envelope, change);
            self.seen_idempotency
                .insert(idempotency_key, envelope.checksum);
        }
    }

    fn record_transaction_evidence(
        &mut self,
        envelope: &TransactionEnvelope,
    ) -> Result<(), LakeError> {
        self.seen_transactions
            .insert(RawCdcTransactionKey::new(envelope)?, envelope.checksum);
        Ok(())
    }
}
