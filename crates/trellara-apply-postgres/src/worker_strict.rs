use trellara_protocol::TransactionEnvelope;
use trellara_stream::StreamConsumer;

use crate::barrier::{optional_header, validate_strict_header_context, ApplyStep};
use crate::worker_transaction::validate_apply_outcome_commit_lsn;
use crate::{ApplyRunStats, ApplyWorkerError, ApplyWorkerResult, EnvelopeApplier};

pub struct ApplyWorker<C, A> {
    consumer: C,
    applier: A,
}

impl<C, A> ApplyWorker<C, A>
where
    C: StreamConsumer,
    A: EnvelopeApplier,
{
    pub fn new(consumer: C, applier: A) -> Self {
        Self { consumer, applier }
    }

    pub async fn run_once(&mut self) -> ApplyWorkerResult<Option<ApplyStep>> {
        let Some(message) = self.consumer.next().await? else {
            return Ok(None);
        };

        if let Some(message_kind) = optional_header(&message.headers, "trellara.message_kind")? {
            if message_kind != "strict_transaction" {
                return Err(ApplyWorkerError::UnsupportedMessageKind(message_kind));
            }
        }

        let envelope = TransactionEnvelope::decode_checked(&message.payload)?;
        validate_strict_header_context(&message, &envelope)?;
        let outcome = self.applier.apply_envelope(&envelope).await?;
        validate_apply_outcome_commit_lsn(
            &envelope.transaction_id,
            &envelope.commit_lsn,
            &outcome,
        )?;
        self.consumer.ack(&message).await?;

        Ok(Some(ApplyStep {
            transaction_id: envelope.transaction_id,
            commit_lsn: envelope.commit_lsn,
            decision: outcome.decision,
            applied_changes: outcome.applied_changes,
            acked_messages: 1,
        }))
    }

    pub async fn run_until_idle(&mut self) -> ApplyWorkerResult<ApplyRunStats> {
        let mut stats = ApplyRunStats::default();

        while let Some(step) = self.run_once().await? {
            stats.record_step(step)?;
        }

        Ok(stats)
    }
}
