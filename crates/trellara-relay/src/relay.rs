use trellara_checkpoint::{CheckpointStore, FlowKey};
use trellara_pg_capture::ChangeSource;
use trellara_protocol::{Checkpoint, TransactionEnvelope};
use trellara_stream::StreamPublisher;

use crate::{publish, RelayMode, RelayRunStats, RelayStep, Result};

pub struct Relay<S, P, C> {
    source: S,
    publisher: P,
    checkpoint_store: C,
    mode: RelayMode,
}

impl<S, P, C> Relay<S, P, C>
where
    S: ChangeSource + Send,
    P: StreamPublisher,
    C: CheckpointStore,
{
    pub fn new(source: S, publisher: P, checkpoint_store: C) -> Self {
        Self::with_mode(source, publisher, checkpoint_store, RelayMode::Strict)
    }

    pub fn with_mode(source: S, publisher: P, checkpoint_store: C, mode: RelayMode) -> Self {
        Self {
            source,
            publisher,
            checkpoint_store,
            mode,
        }
    }

    pub async fn run_once(&mut self) -> Result<Option<RelayStep>> {
        let Some(envelope) = self.source.next_transaction().await? else {
            return Ok(None);
        };

        let step = self.publish_envelope(envelope).await?;
        self.source
            .acknowledge_durable_lsn(&step.source_ack_lsn)
            .await?;
        Ok(Some(step))
    }

    pub async fn run_until_idle(&mut self) -> Result<RelayRunStats> {
        let mut stats = RelayRunStats::default();

        while let Some(step) = self.run_once().await? {
            stats.record_step(step)?;
        }

        Ok(stats)
    }

    async fn publish_envelope(&self, envelope: TransactionEnvelope) -> Result<RelayStep> {
        publish::publish_envelope(
            &self.publisher,
            &self.checkpoint_store,
            &self.mode,
            envelope,
        )
        .await
    }
}

pub async fn load_source_checkpoint<C>(
    checkpoint_store: &C,
    source_id: &str,
    dataset_id: &str,
) -> Result<Option<Checkpoint>>
where
    C: CheckpointStore,
{
    Ok(checkpoint_store
        .load_checkpoint(&FlowKey::new(source_id, dataset_id))
        .await?)
}
