use std::collections::HashMap;

use trellara_stream::StreamConsumer;

use crate::barrier::{ApplyStep, PendingBarrierTransaction};
use crate::worker_ack_messages::ordered_ack_messages;
use crate::worker_ready_envelope::build_ready_envelope;
use crate::{ApplyWorkerError, ApplyWorkerResult, EnvelopeApplier};

pub(crate) async fn apply_ready_transaction<C, A>(
    consumer: &mut C,
    applier: &mut A,
    pending: &mut HashMap<String, PendingBarrierTransaction>,
    transaction_key: &str,
) -> ApplyWorkerResult<ApplyStep>
where
    C: StreamConsumer,
    A: EnvelopeApplier,
{
    let pending_transaction =
        pending
            .get(transaction_key)
            .ok_or_else(|| ApplyWorkerError::ReadyTransactionMissing {
                transaction_key: transaction_key.to_string(),
            })?;
    let envelope = build_ready_envelope(pending_transaction, transaction_key)?;
    let ack_messages = ordered_ack_messages(pending_transaction, transaction_key)?;

    let outcome = applier.apply_envelope(&envelope).await?;
    validate_apply_outcome_commit_lsn(&envelope.transaction_id, &envelope.commit_lsn, &outcome)?;

    for message in &ack_messages {
        consumer.ack(message).await?;
    }

    pending.remove(transaction_key);

    Ok(ApplyStep {
        transaction_id: envelope.transaction_id,
        commit_lsn: envelope.commit_lsn,
        decision: outcome.decision,
        applied_changes: outcome.applied_changes,
        acked_messages: ack_messages.len(),
    })
}

pub(crate) fn validate_apply_outcome_commit_lsn(
    transaction_id: &str,
    expected_commit_lsn: &str,
    outcome: &crate::ApplyOutcome,
) -> ApplyWorkerResult<()> {
    if outcome.commit_lsn == expected_commit_lsn {
        Ok(())
    } else {
        Err(ApplyWorkerError::ApplyOutcomeCommitLsnMismatch {
            transaction_id: transaction_id.to_string(),
            expected: expected_commit_lsn.to_string(),
            actual: outcome.commit_lsn.clone(),
        })
    }
}
