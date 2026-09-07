use std::path::Path;

use trellara_apply_postgres::PostgresApplier;
use trellara_runtime::RuntimeService;
use trellara_stream::StreamConsumer;
#[cfg(feature = "kafka")]
use trellara_stream_kafka::KafkaConsumer;
#[cfg(feature = "local-stream")]
use trellara_stream_local::LocalConsumer;

use crate::{
    barrier_worker, strict_worker, supervise_service, ApplyArgs, ApplyServiceStep, ApplySummary,
    ContinuousWorker, Result, ServiceRuntime, ServiceRuntimeOptions, StreamConfig, TrellaraConfig,
};

pub(crate) async fn run_continuous_apply(
    args: &ApplyArgs,
    initial: &TrellaraConfig,
) -> Result<ApplySummary> {
    initial.validate()?;
    initial.target.as_ref().ok_or_else(|| {
        crate::CliError::InvalidConfig("target.database_url is required".to_string())
    })?;
    let options = ServiceRuntimeOptions::from_args(&args.service)?;
    let runtime = ServiceRuntime::start(
        RuntimeService::Applier,
        &initial.source.id,
        &initial.dataset.id,
        args.health_listen,
        options,
    )
    .await?;
    let path = args.config.clone();
    let source_id = initial.source.id.clone();
    let dataset_id = initial.dataset.id.clone();
    Ok(supervise_service(
        runtime,
        move || {
            let path = path.clone();
            let source_id = source_id.clone();
            let dataset_id = dataset_id.clone();
            async move { build_apply_worker(&path, &source_id, &dataset_id).await }
        },
        ApplySummary::default(),
        record_apply_step,
    )
    .await)
}

fn record_apply_step(
    summary: &mut ApplySummary,
    service_step: ApplyServiceStep,
) -> std::result::Result<Option<String>, String> {
    match service_step {
        ApplyServiceStep::Applied { step, pending } => {
            let lsn = step.commit_lsn.clone();
            summary
                .record_step(step)
                .map_err(|_| "apply stats failed")?;
            summary.record_barrier_pending(pending);
            Ok(Some(lsn))
        }
        ApplyServiceStep::Buffered { pending } => {
            summary.record_barrier_pending(pending);
            Ok(None)
        }
    }
}

async fn build_apply_worker(
    path: &Path,
    source_id: &str,
    dataset_id: &str,
) -> std::result::Result<Box<dyn ContinuousWorker<Step = ApplyServiceStep>>, String> {
    let config = TrellaraConfig::from_path(path).map_err(|_| "config reload failed")?;
    config.validate().map_err(|_| "config validation failed")?;
    if config.source.id != source_id || config.dataset.id != dataset_id {
        return Err("config reload cannot change source.id or dataset.id".to_string());
    }
    let target = config
        .target
        .as_ref()
        .ok_or_else(|| "target.database_url is required".to_string())?;
    let applier = PostgresApplier::connect(
        config
            .to_postgres_apply_config(target.database_url.expose().to_string(), true)
            .map_err(|_| "applier config failed")?,
    )
    .await
    .map_err(|_| "target connect failed")?;
    match &config.stream {
        StreamConfig::Kafka { .. } => {
            #[cfg(feature = "kafka")]
            {
                let consumer = KafkaConsumer::new(
                    config
                        .to_kafka_consumer_config()
                        .map_err(|_| "consumer config failed")?,
                )
                .map_err(|_| "consumer connect failed")?;
                Ok(worker_for_mode(consumer, applier, &config))
            }
            #[cfg(not(feature = "kafka"))]
            {
                Err(crate::kafka_feature_disabled().to_string())
            }
        }
        StreamConfig::Local { .. } => {
            #[cfg(feature = "local-stream")]
            {
                let consumer = LocalConsumer::new(
                    config
                        .to_local_consumer_config()
                        .map_err(|_| "consumer config failed")?,
                )
                .map_err(|_| "consumer connect failed")?;
                Ok(worker_for_mode(consumer, applier, &config))
            }
            #[cfg(not(feature = "local-stream"))]
            {
                Err(crate::local_stream_feature_disabled().to_string())
            }
        }
    }
}

fn worker_for_mode<C>(
    consumer: C,
    applier: PostgresApplier,
    config: &TrellaraConfig,
) -> Box<dyn ContinuousWorker<Step = ApplyServiceStep>>
where
    C: StreamConsumer + 'static,
{
    if config.uses_barrier_apply() {
        barrier_worker(consumer, applier)
    } else {
        strict_worker(consumer, applier)
    }
}
