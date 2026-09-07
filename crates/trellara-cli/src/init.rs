use std::fs;

use serde::Serialize;

use crate::{
    fleet_init_next_commands, init_fleet_plan::plan_fleet_init_configs, render_fleet_init_readme,
    CliError, FleetInitArgs, Result,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetInitSummary {
    pub(crate) fleet_id: String,
    pub(crate) output_dir: String,
    pub(crate) flow_count: usize,
    pub(crate) config_count: usize,
    pub(crate) manifest: String,
    pub(crate) readme: String,
    pub(crate) configs: Vec<FleetInitConfigSummary>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetInitConfigSummary {
    pub(crate) dataset_id: String,
    pub(crate) config: String,
    pub(crate) stream_path: String,
    pub(crate) source_slot: String,
    pub(crate) table_count: usize,
}

pub(crate) fn init_fleet_configs(args: &FleetInitArgs) -> Result<FleetInitSummary> {
    let output_dir = &args.output_dir;
    let config_dir = output_dir.join("configs");
    let stream_dir = output_dir.join("streams");
    let manifest_path = output_dir.join("fleet-manifest.json");
    let readme_path = output_dir.join("README.md");
    let plan = plan_fleet_init_configs(args)?;

    fs::create_dir_all(&config_dir).map_err(|source| CliError::WriteOutput {
        path: config_dir.display().to_string(),
        source,
    })?;
    fs::create_dir_all(&stream_dir).map_err(|source| CliError::WriteOutput {
        path: stream_dir.display().to_string(),
        source,
    })?;

    for (config_path, yaml) in plan.config_paths.iter().zip(plan.rendered_configs.iter()) {
        fs::write(config_path, yaml).map_err(|source| CliError::WriteOutput {
            path: config_path.display().to_string(),
            source,
        })?;
    }

    let next_commands = fleet_init_next_commands(&plan.configs);
    let summary = FleetInitSummary {
        fleet_id: args.fleet_id.clone(),
        output_dir: output_dir.display().to_string(),
        flow_count: plan.configs.len(),
        config_count: plan.configs.len(),
        manifest: manifest_path.display().to_string(),
        readme: readme_path.display().to_string(),
        configs: plan.configs,
        next_commands,
    };

    fs::create_dir_all(output_dir).map_err(|source| CliError::WriteOutput {
        path: output_dir.display().to_string(),
        source,
    })?;
    fs::write(&manifest_path, serde_json::to_string_pretty(&summary)?).map_err(|source| {
        CliError::WriteOutput {
            path: manifest_path.display().to_string(),
            source,
        }
    })?;
    fs::write(&readme_path, render_fleet_init_readme(&summary)).map_err(|source| {
        CliError::WriteOutput {
            path: readme_path.display().to_string(),
            source,
        }
    })?;

    Ok(summary)
}
