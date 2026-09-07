use std::path::Path;

use trellara_checkpoint::PostgresCheckpointStore;
use trellara_pg_capture::{ChangeSource, PgOutputStreamCapture, TestDecodingCapture};
use trellara_relay::{Relay, RelayStep};
use trellara_runtime::RuntimeService;
use trellara_stream::StreamPublisher;
#[cfg(feature = "kafka")]
use trellara_stream_kafka::KafkaPublisher;
#[cfg(feature = "local-stream")]
use trellara_stream_local::LocalPublisher;

use crate::{
    supervise_service, ContinuousWorker, RelayArgs, RelaySummary, Result, ServiceRuntime,
    ServiceRuntimeOptions, SourceCaptureKind, StreamConfig, TrellaraConfig, WorkerFuture,
};

pub(crate) async fn run_continuous_relay(
    args: &RelayArgs,
    initial: &TrellaraConfig,
) -> Result<RelaySummary> {
    initial.validate()?;
    let options = ServiceRuntimeOptions::from_args(&args.service)?;
    let runtime = ServiceRuntime::start(
        RuntimeService::Relay,
        &initial.source.id,
        &initial.dataset.id,
        args.health_listen,
        options,
    )
    .await?;
    let path = args.config.clone();
    let source_id = initial.source.id.clone();
    let dataset_id = initial.dataset.id.clone();
    let create_if_missing = args.create_if_missing;
    Ok(
        supervise_service(
            runtime,
            move || {
                let path = path.clone();
                let source_id = source_id.clone();
                let dataset_id = dataset_id.clone();
                async move {
                    build_relay_worker(&path, &source_id, &dataset_id, create_if_missing).await
                }
            },
            RelaySummary::default(),
            |summary, step: RelayStep| {
                let lsn = step.envelope.commit_lsn.clone();
                summary
                    .record_step(step)
                    .map_err(|_| "relay stats failed".to_string())?;
                Ok(Some(lsn))
            },
        )
        .await,
    )
}

async fn build_relay_worker(
    path: &Path,
    source_id: &str,
    dataset_id: &str,
    create_if_missing: bool,
) -> std::result::Result<Box<dyn ContinuousWorker<Step = RelayStep>>, String> {
    let config = TrellaraConfig::from_path(path).map_err(|_| "config reload failed")?;
    config.validate().map_err(|_| "config validation failed")?;
    validate_identity(&config, source_id, dataset_id)?;
    let capture_config = config
        .to_capture_config(create_if_missing)
        .map_err(|_| "capture config failed")?;
    match config.source.capture {
        SourceCaptureKind::PgOutput => {
            let source = PgOutputStreamCapture::connect(capture_config)
                .await
                .map_err(|_| "source connect failed")?;
            build_relay_with_source(source, config).await
        }
        SourceCaptureKind::TestDecoding => {
            let source = TestDecodingCapture::connect(capture_config)
                .await
                .map_err(|_| "source connect failed")?;
            build_relay_with_source(source, config).await
        }
    }
}

async fn build_relay_with_source<S>(
    source: S,
    config: TrellaraConfig,
) -> std::result::Result<Box<dyn ContinuousWorker<Step = RelayStep>>, String>
where
    S: ChangeSource + Send + 'static,
{
    match &config.stream {
        StreamConfig::Kafka { .. } => {
            #[cfg(feature = "kafka")]
            {
                let publisher = KafkaPublisher::new(
                    config
                        .to_kafka_publisher_config()
                        .map_err(|_| "publisher config failed")?,
                )
                .map_err(|_| "publisher connect failed")?;
                boxed_relay(source, publisher, &config).await
            }
            #[cfg(not(feature = "kafka"))]
            {
                Err(crate::kafka_feature_disabled().to_string())
            }
        }
        StreamConfig::Local { .. } => {
            #[cfg(feature = "local-stream")]
            {
                let publisher = LocalPublisher::new(
                    config
                        .to_local_publisher_config()
                        .map_err(|_| "publisher config failed")?,
                )
                .map_err(|_| "publisher connect failed")?;
                boxed_relay(source, publisher, &config).await
            }
            #[cfg(not(feature = "local-stream"))]
            {
                Err(crate::local_stream_feature_disabled().to_string())
            }
        }
    }
}

async fn boxed_relay<S, P>(
    source: S,
    publisher: P,
    config: &TrellaraConfig,
) -> std::result::Result<Box<dyn ContinuousWorker<Step = RelayStep>>, String>
where
    S: ChangeSource + Send + 'static,
    P: StreamPublisher + 'static,
{
    let checkpoint = PostgresCheckpointStore::connect(&config.source.database_url, true)
        .await
        .map_err(|_| "checkpoint connect failed")?;
    let mode = config.to_relay_mode().map_err(|_| "relay mode failed")?;
    Ok(Box::new(RelayServiceWorker {
        relay: Relay::with_mode(source, publisher, checkpoint, mode),
    }))
}

struct RelayServiceWorker<S, P> {
    relay: Relay<S, P, PostgresCheckpointStore>,
}

impl<S, P> ContinuousWorker for RelayServiceWorker<S, P>
where
    S: ChangeSource + Send,
    P: StreamPublisher,
{
    type Step = RelayStep;

    fn run_once(&mut self) -> WorkerFuture<'_, RelayStep> {
        Box::pin(async {
            self.relay
                .run_once()
                .await
                .map_err(|_| "relay worker failed".to_string())
        })
    }
}

fn validate_identity(
    config: &TrellaraConfig,
    source_id: &str,
    dataset_id: &str,
) -> std::result::Result<(), String> {
    if config.source.id != source_id || config.dataset.id != dataset_id {
        Err("config reload cannot change source.id or dataset.id".to_string())
    } else {
        Ok(())
    }
}
