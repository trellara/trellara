use std::fs;

use serde::Serialize;

use crate::{render_init_config, CliError, InitArgs, Result, TrellaraConfig};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct InitSummary {
    pub(crate) config: String,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) table_count: usize,
    pub(crate) stream_kind: String,
    pub(crate) evaluation_ready: bool,
    pub(crate) next_commands: Vec<String>,
}

pub(crate) fn init_flow_config(args: &InitArgs) -> Result<String> {
    let yaml = render_init_config(args)?;
    let config = TrellaraConfig::from_yaml(&yaml, &args.output.display().to_string())?;
    config.validate()?;
    if args.output.exists() && !args.force {
        return Err(CliError::InvalidConfig(format!(
            "output {} already exists; pass --force to overwrite",
            args.output.display()
        )));
    }
    if let Some(parent) = args
        .output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| CliError::WriteOutput {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(&args.output, yaml).map_err(|source| CliError::WriteOutput {
        path: args.output.display().to_string(),
        source,
    })?;

    Ok(serde_json::to_string_pretty(&InitSummary {
        config: args.output.display().to_string(),
        source_id: config.source.id,
        dataset_id: config.dataset.id,
        table_count: config.dataset.tables.len(),
        stream_kind: "local".to_string(),
        evaluation_ready: args.evaluate,
        next_commands: init_next_commands(args),
    })?)
}

fn init_next_commands(args: &InitArgs) -> Vec<String> {
    let config = args.output.display();
    let mut commands = vec![
        format!("trellara check --config {config}"),
        format!("trellara preflight --config {config}"),
        format!(
            "trellara run --local --verify --format text --config {config} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"
        ),
        format!("trellara verify --config {config}"),
        format!("trellara status --config {config}"),
    ];

    if args.evaluate {
        commands.extend([
            format!("trellara quickstart --config {config} --check --format text"),
            format!("trellara status --config {config} --view report --format text"),
            format!("trellara evaluate --config {config} --format text"),
            format!("trellara pilot-package --config {config}"),
        ]);
    }

    commands
}
