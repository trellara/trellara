use std::fs;

use crate::{
    quickstart_capture_spill_message, quickstart_transaction_boundary_message,
    validate_local_run_config, QuickstartArgs, QuickstartReadinessCheck,
    QuickstartReadinessSummary, Result, SourceCaptureKind, StreamConfig, TrellaraConfig,
};

pub(crate) fn quickstart_readiness(args: &QuickstartArgs) -> Result<QuickstartReadinessSummary> {
    let config_path = args.config.display().to_string();
    let mut checks = Vec::new();
    let yaml = match fs::read_to_string(&args.config) {
        Ok(yaml) => {
            checks.push(QuickstartReadinessCheck::passed(
                "config_readable",
                format!("read config {config_path}"),
            ));
            yaml
        }
        Err(error) => {
            checks.push(QuickstartReadinessCheck::failed(
                "config_readable",
                format!("could not read config {config_path}: {error}"),
                Some(format!(
                    "run trellara quickstart --config {config_path}, then run the generated trellara init command"
                )),
            ));
            return Ok(QuickstartReadinessSummary::from_checks(config_path, checks));
        }
    };

    let config = match TrellaraConfig::from_yaml(&yaml, &config_path) {
        Ok(config) => {
            checks.push(QuickstartReadinessCheck::passed(
                "config_parse",
                "config YAML parsed successfully",
            ));
            config
        }
        Err(error) => {
            checks.push(QuickstartReadinessCheck::failed(
                "config_parse",
                format!("config YAML is invalid: {error}"),
                Some("rerun trellara init or repair the YAML syntax".to_string()),
            ));
            return Ok(QuickstartReadinessSummary::from_checks(config_path, checks));
        }
    };

    match config.validate() {
        Ok(()) => checks.push(QuickstartReadinessCheck::passed(
            "config_validate",
            "config contract is valid",
        )),
        Err(error) => checks.push(QuickstartReadinessCheck::failed(
            "config_validate",
            format!("config contract is invalid: {error}"),
            Some(
                "run trellara validate --config <config> for the full validation error".to_string(),
            ),
        )),
    }

    if matches!(config.stream, StreamConfig::Local { .. }) {
        checks.push(QuickstartReadinessCheck::passed(
            "local_stream",
            "stream.kind is local for the no-broker demo",
        ));
    } else {
        checks.push(QuickstartReadinessCheck::failed(
            "local_stream",
            "stream.kind is not local; quickstart demo is brokerless",
            Some("use trellara init defaults or set stream.kind: local".to_string()),
        ));
    }

    if config.target.is_some() {
        checks.push(QuickstartReadinessCheck::passed(
            "target_database",
            "target.database_url is configured",
        ));
    } else {
        checks.push(QuickstartReadinessCheck::failed(
            "target_database",
            "target.database_url is required for trellara run --local",
            Some("rerun trellara init with --target-database-url".to_string()),
        ));
    }

    if config.source.capture == SourceCaptureKind::PgOutput {
        checks.push(QuickstartReadinessCheck::passed(
            "pgoutput_capture",
            "source capture uses pgoutput",
        ));
    } else {
        checks.push(QuickstartReadinessCheck::failed(
            "pgoutput_capture",
            "quickstart should use pgoutput, not test_decoding",
            Some("remove source.capture: test_decoding or rerun trellara init".to_string()),
        ));
    }

    checks.push(QuickstartReadinessCheck::passed(
        "transaction_boundary",
        quickstart_transaction_boundary_message(&config),
    ));

    checks.push(QuickstartReadinessCheck::passed(
        "capture_spill_boundary",
        quickstart_capture_spill_message(&config),
    ));

    if validate_local_run_config(&config).is_ok() {
        checks.push(QuickstartReadinessCheck::passed(
            "local_run_ready",
            "trellara run --local --verify can run this config",
        ));
    } else {
        checks.push(QuickstartReadinessCheck::failed(
            "local_run_ready",
            "trellara run --local --verify is blocked until the failed checks above are fixed",
            Some(format!("trellara quickstart --config {config_path}")),
        ));
    }

    Ok(QuickstartReadinessSummary::from_checks(config_path, checks))
}
