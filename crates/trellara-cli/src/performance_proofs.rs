use crate::{DatasetMode, StreamConfig, TrellaraConfig};

pub(crate) fn performance_proof_commands(
    config: &TrellaraConfig,
    config_path: &str,
) -> Vec<String> {
    let mut commands = vec![
        format!("trellara performance --config {config_path} --format text"),
        format!("trellara quickstart --config {config_path} --check --format text"),
        performance_run_proof_command(config, config_path),
        format!("trellara status --config {config_path} --view metrics"),
        format!("trellara status --config {config_path} --view report --format text"),
        "trellara chaos run".to_string(),
    ];
    if matches!(config.stream, StreamConfig::Local { .. }) {
        commands.push(format!(
            "trellara stream inspect-local --config {config_path}"
        ));
    }
    if config.dataset.mode == DatasetMode::PartitionedScaleMode {
        commands.push(format!(
            "trellara partition-watermarks --config {config_path}"
        ));
    }
    commands
}

fn performance_run_proof_command(config: &TrellaraConfig, config_path: &str) -> String {
    if matches!(config.stream, StreamConfig::Local { .. }) {
        format!(
            "trellara run --local --verify --format text --config {config_path} --snapshot-run-id performance-envelope --max-transactions 100 --max-messages 100"
        )
    } else {
        format!(
            "trellara run --verify --format text --config {config_path} --snapshot-run-id performance-envelope --max-transactions 100 --max-messages 100"
        )
    }
}
