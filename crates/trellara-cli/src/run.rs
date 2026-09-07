#[cfg(feature = "local-stream")]
use trellara_apply_postgres::PostgresApplier;
#[cfg(feature = "local-stream")]
use trellara_pg_capture::{PgChangeSource, PgOutputStreamCapture, TestDecodingCapture};
#[cfg(feature = "local-stream")]
use trellara_stream_local::LocalConsumer;

#[cfg(feature = "local-stream")]
use crate::{
    bootstrap_configured_capture, mark_snapshot_summary_verified, render_run_summary,
    run_apply_with_consumer, run_proof_chain, run_relay, snapshot_configured_tables,
    verify_configured_tables, RunSummary, SourceCaptureKind,
};
use crate::{CliError, Result, RunArgs, StreamConfig, TrellaraConfig};

#[cfg(feature = "local-stream")]
pub(crate) async fn run_configured_flow(args: &RunArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    validate_local_run_config(&config)?;
    let target = config
        .target
        .clone()
        .expect("validated local run target database url");

    let applier = PostgresApplier::connect(
        config.to_postgres_apply_config(target.database_url.expose().to_string(), true)?,
    )
    .await?;
    applier.ensure_schema().await?;

    let mut snapshot = if args.skip_snapshot {
        None
    } else {
        Some(
            snapshot_configured_tables(
                &config,
                &target.database_url,
                Some(&args.snapshot_run_id),
                None,
                args.create_if_missing,
                false,
            )
            .await?,
        )
    };
    let capture_config = config.to_capture_config(args.create_if_missing)?;
    let (bootstrap, relay) = match config.source.capture {
        SourceCaptureKind::PgOutput => {
            let mut capture = PgOutputStreamCapture::connect(capture_config).await?;
            let bootstrap =
                crate::capture_bootstrap_summary(&config, capture.bootstrap().await?).await?;
            let relay = run_relay(capture, &config, args.max_transactions).await?;
            (bootstrap, relay)
        }
        SourceCaptureKind::TestDecoding => {
            let bootstrap = bootstrap_configured_capture(&config, args.create_if_missing).await?;
            let capture = TestDecodingCapture::connect(capture_config).await?;
            let relay = run_relay(capture, &config, args.max_transactions).await?;
            (bootstrap, relay)
        }
    };
    let consumer = LocalConsumer::new(config.to_local_consumer_config()?)?;
    let apply = run_apply_with_consumer(
        consumer,
        &config,
        target.database_url.expose().to_string(),
        args.max_messages,
    )
    .await?;
    let verify = if args.verify {
        Some(verify_configured_tables(&config, &target.database_url, None).await?)
    } else {
        None
    };
    mark_snapshot_summary_verified(snapshot.as_mut(), verify.as_ref());
    let mut relay = relay;
    relay.local_stream_evidence = Some(crate::collect_local_run_stream_evidence(
        &config,
        &relay.latest_publish_messages,
        &relay.latest_publish_acks,
        relay.last_topic.as_deref(),
        relay.last_partition,
        relay.last_offset,
    )?);

    let mut next_commands = Vec::new();
    if verify.is_none() {
        next_commands.push(format!(
            "trellara verify --config {}",
            args.config.display()
        ));
    }
    next_commands.push(format!(
        "trellara status --config {}",
        args.config.display()
    ));
    next_commands.push(format!(
        "trellara status --config {} --view report --format text",
        args.config.display()
    ));

    let proof_chain = run_proof_chain(
        &args.config,
        &config,
        snapshot.as_ref(),
        &bootstrap,
        &relay,
        &apply,
        verify.as_ref(),
    );

    let summary = RunSummary {
        config: args.config.display().to_string(),
        source_id: config.source.id,
        dataset_id: config.dataset.id,
        mode: config.dataset.mode.to_string(),
        stream_kind: "local".to_string(),
        bootstrap,
        apply_schema_ready: true,
        snapshot,
        relay,
        apply,
        verify,
        proof_chain,
        next_commands,
    };

    render_run_summary(&summary, args.format)
}

#[cfg(not(feature = "local-stream"))]
pub(crate) async fn run_configured_flow(args: &RunArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    validate_local_run_config(&config)?;
    Err(crate::local_stream_feature_disabled())
}

pub(crate) fn validate_local_run_config(config: &TrellaraConfig) -> Result<()> {
    config.validate()?;
    if !matches!(config.stream, StreamConfig::Local { .. }) {
        return Err(CliError::InvalidConfig(
            "trellara run currently requires stream.kind: local; use trellara relay and trellara apply for Kafka-backed flows".to_string(),
        ));
    }
    if config.target.is_none() {
        return Err(CliError::InvalidConfig(
            "target.database_url is required for trellara run".to_string(),
        ));
    }
    Ok(())
}
