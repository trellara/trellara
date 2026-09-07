use trellara_apply_postgres::{ApplyWorker, BarrierAwareApplyWorker, PostgresApplier};
use trellara_checkpoint::PostgresCheckpointStore;
use trellara_pg_capture::ChangeSource;
use trellara_relay::Relay;
use trellara_stream::{StreamConsumer, StreamPublisher};
#[cfg(feature = "kafka")]
use trellara_stream_kafka::KafkaPublisher;
#[cfg(feature = "local-stream")]
use trellara_stream_local::LocalPublisher;

use crate::{ApplySummary, DatasetMode, RelaySummary, Result, StreamConfig, TrellaraConfig};

pub(crate) async fn run_relay<S>(
    source: S,
    config: &TrellaraConfig,
    max_transactions: u64,
) -> Result<RelaySummary>
where
    S: ChangeSource + Send,
{
    match &config.stream {
        StreamConfig::Kafka { .. } => {
            #[cfg(feature = "kafka")]
            {
                let publisher = KafkaPublisher::new(config.to_kafka_publisher_config()?)?;
                run_relay_with_publisher(source, publisher, config, max_transactions).await
            }
            #[cfg(not(feature = "kafka"))]
            {
                Err(crate::kafka_feature_disabled())
            }
        }
        StreamConfig::Local { .. } => {
            #[cfg(feature = "local-stream")]
            {
                let publisher = LocalPublisher::new(config.to_local_publisher_config()?)?;
                run_relay_with_publisher(source, publisher, config, max_transactions).await
            }
            #[cfg(not(feature = "local-stream"))]
            {
                Err(crate::local_stream_feature_disabled())
            }
        }
    }
}

pub(crate) async fn run_relay_with_publisher<S, P>(
    source: S,
    publisher: P,
    config: &TrellaraConfig,
    max_transactions: u64,
) -> Result<RelaySummary>
where
    S: ChangeSource + Send,
    P: StreamPublisher,
{
    let checkpoint = PostgresCheckpointStore::connect(&config.source.database_url, true).await?;
    let mut relay = Relay::with_mode(source, publisher, checkpoint, config.to_relay_mode()?);

    if max_transactions == 0 {
        Ok(RelaySummary::from_stats(relay.run_until_idle().await?))
    } else {
        let mut summary = RelaySummary::default();
        for _ in 0..max_transactions {
            let Some(step) = relay.run_once().await? else {
                break;
            };
            summary.record_step(step)?;
        }
        Ok(summary)
    }
}

pub(crate) async fn run_apply_with_consumer<C>(
    consumer: C,
    config: &TrellaraConfig,
    target_database_url: String,
    max_messages: u64,
) -> Result<ApplySummary>
where
    C: StreamConsumer,
{
    let applier =
        PostgresApplier::connect(config.to_postgres_apply_config(target_database_url, true)?)
            .await?;
    let summary = if config.uses_barrier_apply() {
        let mut worker = BarrierAwareApplyWorker::new(consumer, applier);
        if max_messages == 0 {
            ApplySummary::from_stats(worker.run_until_idle().await?)
        } else {
            let mut summary = ApplySummary::default();
            for _ in 0..max_messages {
                let Some(step) = worker.run_once().await? else {
                    break;
                };
                if let trellara_apply_postgres::BarrierApplyStep::Applied(step) = step {
                    summary.record_step(step)?;
                }
            }
            summary.record_barrier_pending(worker.pending_barrier_stats());
            summary
        }
    } else {
        match config.dataset.mode {
            DatasetMode::StrictTransactionOrder => {
                let mut worker = ApplyWorker::new(consumer, applier);
                if max_messages == 0 {
                    ApplySummary::from_stats(worker.run_until_idle().await?)
                } else {
                    let mut summary = ApplySummary::default();
                    for _ in 0..max_messages {
                        let Some(step) = worker.run_once().await? else {
                            break;
                        };
                        summary.record_step(step)?;
                    }
                    summary
                }
            }
            DatasetMode::PartitionedScaleMode => {
                unreachable!("partitioned mode uses barrier apply")
            }
        }
    };

    Ok(summary)
}
