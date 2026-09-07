use trellara_pg_capture::{PgCapture, PgOutputStreamCapture, TestDecodingCapture};

use crate::*;

pub(crate) async fn execute_contract_command(command: ContractCommand) -> Result<String> {
    match command {
        ContractCommand::Test(args) => contract_test_command(args).await,
    }
}

pub(crate) async fn contract_test_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    let capture = PgCapture::connect(config.to_capture_config(false)?).await?;
    let preflight = apply_preflight_contracts(&config, capture.inspect_tables().await?).await?;

    Ok(serde_json::to_string_pretty(
        &ContractTestSummary::from_preflight(&config, preflight),
    )?)
}

pub(crate) async fn preflight_command(args: ConfigArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    let capture = PgCapture::connect(config.to_capture_config(false)?).await?;
    let preflight = apply_preflight_contracts(&config, capture.inspect_tables().await?).await?;
    Ok(serde_json::to_string_pretty(
        &PreflightSummary::from_tables(preflight),
    )?)
}

pub(crate) async fn bootstrap_command(args: BootstrapArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    Ok(serde_json::to_string_pretty(
        &bootstrap_configured_capture(&config, args.create_if_missing).await?,
    )?)
}

pub(crate) async fn relay_command(args: RelayArgs) -> Result<String> {
    let config = TrellaraConfig::from_path(&args.config)?;
    if args.max_transactions == 0 {
        return Ok(serde_json::to_string_pretty(
            &run_continuous_relay(&args, &config).await?,
        )?);
    }
    let capture_config = config.to_capture_config(args.create_if_missing)?;
    let summary = match config.source.capture {
        SourceCaptureKind::PgOutput => {
            let capture = PgOutputStreamCapture::connect(capture_config).await?;
            run_relay(capture, &config, args.max_transactions).await?
        }
        SourceCaptureKind::TestDecoding => {
            let capture = TestDecodingCapture::connect(capture_config).await?;
            run_relay(capture, &config, args.max_transactions).await?
        }
    };

    Ok(serde_json::to_string_pretty(&summary)?)
}
