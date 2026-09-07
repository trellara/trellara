use std::collections::HashMap;

use trellara_protocol::{barrier_visibility_decision, BarrierVisibilityDecision};
use trellara_stream::StreamConsumer;

use crate::barrier::{
    required_header, ApplyStep, BarrierApplyStep, BarrierPendingStats, HeaderContext,
    PendingBarrierTransaction,
};
use crate::worker_buffer::buffer_barrier_message;
use crate::worker_transaction::apply_ready_transaction;
use crate::{ApplyRunStats, ApplyWorkerResult, EnvelopeApplier};

pub struct BarrierAwareApplyWorker<C, A> {
    consumer: C,
    applier: A,
    pending: HashMap<String, PendingBarrierTransaction>,
}

impl<C, A> BarrierAwareApplyWorker<C, A>
where
    C: StreamConsumer,
    A: EnvelopeApplier,
{
    pub fn new(consumer: C, applier: A) -> Self {
        Self {
            consumer,
            applier,
            pending: HashMap::new(),
        }
    }

    pub async fn run_once(&mut self) -> ApplyWorkerResult<Option<BarrierApplyStep>> {
        if let Some(transaction_key) = self.ready_transaction_key()? {
            let step = self.apply_ready_transaction(&transaction_key).await?;
            return Ok(Some(BarrierApplyStep::Applied(step)));
        }

        let Some(message) = self.consumer.next().await? else {
            return Ok(None);
        };

        let context = HeaderContext::from_message(&message)?;
        let transaction_key = context.transaction_key();
        let message_kind = required_header(&message.headers, "trellara.message_kind")?;
        let pending = self.pending.entry(transaction_key.clone()).or_default();

        buffer_barrier_message(pending, context, &message_kind, message)?;

        if self.transaction_ready(&transaction_key)? {
            let step = self.apply_ready_transaction(&transaction_key).await?;
            Ok(Some(BarrierApplyStep::Applied(step)))
        } else {
            Ok(Some(BarrierApplyStep::Buffered { transaction_key }))
        }
    }

    pub async fn run_until_idle(&mut self) -> ApplyWorkerResult<ApplyRunStats> {
        let mut stats = ApplyRunStats::default();

        while let Some(step) = self.run_once().await? {
            if let BarrierApplyStep::Applied(applied_step) = step {
                stats.record_step(applied_step)?;
            }
        }
        stats.barrier_pending = self.pending_barrier_stats();

        Ok(stats)
    }

    pub fn pending_barrier_stats(&self) -> BarrierPendingStats {
        BarrierPendingStats::from_pending(self.pending.values())
    }

    fn transaction_ready(&self, transaction_key: &str) -> ApplyWorkerResult<bool> {
        let Some(pending) = self.pending.get(transaction_key) else {
            return Ok(false);
        };
        let Some(manifest) = &pending.manifest else {
            return Ok(false);
        };
        let chunks = pending
            .chunks
            .values()
            .map(|pending_chunk| pending_chunk.chunk.clone())
            .collect::<Vec<_>>();
        let marker = pending
            .commit_marker
            .as_ref()
            .map(|pending_marker| &pending_marker.marker);

        match barrier_visibility_decision(&manifest.manifest, marker, &chunks)? {
            BarrierVisibilityDecision::GloballyVisible { .. } => Ok(true),
            BarrierVisibilityDecision::Held { .. } => Ok(false),
        }
    }

    fn ready_transaction_key(&self) -> ApplyWorkerResult<Option<String>> {
        let mut transaction_keys = self.pending.keys().collect::<Vec<_>>();
        transaction_keys.sort();
        for transaction_key in transaction_keys {
            if self.transaction_ready(transaction_key)? {
                return Ok(Some(transaction_key.clone()));
            }
        }
        Ok(None)
    }

    async fn apply_ready_transaction(
        &mut self,
        transaction_key: &str,
    ) -> ApplyWorkerResult<ApplyStep> {
        apply_ready_transaction(
            &mut self.consumer,
            &mut self.applier,
            &mut self.pending,
            transaction_key,
        )
        .await
    }
}
