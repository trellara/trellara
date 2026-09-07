use trellara_apply_postgres::PostgresApplier;
#[cfg(feature = "kafka")]
use trellara_stream_kafka::KafkaConsumer;
#[cfg(feature = "local-stream")]
use trellara_stream_local::LocalConsumer;

use crate::*;

pub(crate) async fn apply_schema_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let target = config
        .target
        .as_ref()
        .ok_or_else(|| CliError::InvalidConfig("target.database_url is required".to_string()))?;
    let applier = PostgresApplier::connect(
        config.to_postgres_apply_config(target.database_url.expose().to_string(), true)?,
    )
    .await?;
    applier.ensure_schema().await?;
    Ok("target checkpoint and dedup schema is ready".to_string())
}

pub(crate) async fn apply_command(args: ApplyArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    if args.max_messages == 0 {
        return Ok(serde_json::to_string_pretty(
            &run_continuous_apply(&args, &config).await?,
        )?);
    }
    #[cfg(not(any(feature = "kafka", feature = "local-stream")))]
    {
        return match &config.stream {
            StreamConfig::Kafka { .. } => Err(crate::kafka_feature_disabled()),
            StreamConfig::Local { .. } => Err(crate::local_stream_feature_disabled()),
        };
    }
    #[cfg(any(feature = "kafka", feature = "local-stream"))]
    {
        let target = config.target.clone().ok_or_else(|| {
            CliError::InvalidConfig("target.database_url is required".to_string())
        })?;
        let summary = match &config.stream {
            StreamConfig::Kafka { .. } => {
                #[cfg(feature = "kafka")]
                {
                    let consumer = KafkaConsumer::new(config.to_kafka_consumer_config()?)?;
                    run_apply_with_consumer(
                        consumer,
                        &config,
                        target.database_url.expose().to_string(),
                        args.max_messages,
                    )
                    .await?
                }
                #[cfg(not(feature = "kafka"))]
                {
                    return Err(crate::kafka_feature_disabled());
                }
            }
            StreamConfig::Local { .. } => {
                #[cfg(feature = "local-stream")]
                {
                    let consumer = LocalConsumer::new(config.to_local_consumer_config()?)?;
                    run_apply_with_consumer(
                        consumer,
                        &config,
                        target.database_url.expose().to_string(),
                        args.max_messages,
                    )
                    .await?
                }
                #[cfg(not(feature = "local-stream"))]
                {
                    return Err(crate::local_stream_feature_disabled());
                }
            }
        };

        Ok(serde_json::to_string_pretty(&summary)?)
    }
}

pub(crate) async fn snapshot_command(args: SnapshotArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    let target = config
        .target
        .clone()
        .ok_or_else(|| CliError::InvalidConfig("target.database_url is required".to_string()))?;
    let summary = snapshot_configured_tables(
        &config,
        &target.database_url,
        args.run_id.as_deref(),
        args.table.as_deref(),
        args.create_if_missing,
        args.force,
    )
    .await?;

    Ok(serde_json::to_string_pretty(&summary)?)
}

pub(crate) async fn verify_command(args: VerifyArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let target = config
        .target
        .clone()
        .ok_or_else(|| CliError::InvalidConfig("target.database_url is required".to_string()))?;
    let summary =
        verify_configured_tables(&config, &target.database_url, args.table.as_deref()).await?;

    Ok(serde_json::to_string_pretty(&summary)?)
}

pub(crate) async fn reseed_command(args: ReseedArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    config.validate()?;
    let target = config
        .target
        .clone()
        .ok_or_else(|| CliError::InvalidConfig("target.database_url is required".to_string()))?;
    let summary =
        reseed_configured_tables(&config, &target.database_url, args.table.as_deref()).await?;

    Ok(serde_json::to_string_pretty(&summary)?)
}
