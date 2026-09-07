use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use crate::{
    config_slug, render_init_config, require_non_empty, CliError, FleetInitArgs,
    FleetInitConfigSummary, InitArgs, Result, TrellaraConfig,
};

pub(crate) struct FleetInitPlan {
    pub(crate) configs: Vec<FleetInitConfigSummary>,
    pub(crate) config_paths: Vec<PathBuf>,
    pub(crate) rendered_configs: Vec<String>,
}

pub(crate) fn plan_fleet_init_configs(args: &FleetInitArgs) -> Result<FleetInitPlan> {
    validate_fleet_init_args(args)?;
    let config_dir = args.output_dir.join("configs");
    let stream_dir = args.output_dir.join("streams");
    let mut seen_dataset_ids = BTreeSet::new();
    let mut seen_slugs = BTreeSet::new();
    let mut configs = Vec::new();
    let mut config_paths = Vec::new();
    let mut rendered_configs = Vec::new();

    for dataset_id in &args.dataset {
        require_non_empty("dataset", dataset_id)?;
        if !seen_dataset_ids.insert(dataset_id.as_str()) {
            return Err(CliError::InvalidConfig(format!(
                "duplicate dataset {dataset_id:?} in fleet init"
            )));
        }
        let slug = config_slug(dataset_id)?;
        if !seen_slugs.insert(slug.clone()) {
            return Err(CliError::InvalidConfig(format!(
                "dataset {dataset_id:?} maps to duplicate config slug {slug:?}"
            )));
        }
        let config_path = config_dir.join(format!("{slug}.yml"));
        let yaml = render_fleet_config(args, dataset_id, &slug, &config_path)?;
        let config = TrellaraConfig::from_yaml(&yaml, &config_path.display().to_string())?;
        config.validate()?;

        configs.push(FleetInitConfigSummary {
            dataset_id: dataset_id.clone(),
            config: config_path.display().to_string(),
            stream_path: stream_dir.join(&slug).display().to_string(),
            source_slot: format!("trellara_{}_slot", slug.replace('-', "_")),
            table_count: config.dataset.tables.len(),
        });
        config_paths.push(config_path);
        rendered_configs.push(yaml);
    }

    Ok(FleetInitPlan {
        configs,
        config_paths,
        rendered_configs,
    })
}

fn validate_fleet_init_args(args: &FleetInitArgs) -> Result<()> {
    require_non_empty("source_database_url", &args.source_database_url)?;
    require_non_empty("fleet_id", &args.fleet_id)?;
    require_non_empty("source_id", &args.source_id)?;
    require_non_empty("database_id", &args.database_id)?;
    require_non_empty("primary_key", &args.primary_key)?;
    if args.dataset.is_empty() {
        return Err(CliError::InvalidConfig(
            "fleet init requires at least one --dataset".to_string(),
        ));
    }
    if args.table.is_empty() {
        return Err(CliError::InvalidConfig(
            "fleet init requires at least one --table schema.table".to_string(),
        ));
    }
    Ok(())
}

fn render_fleet_config(
    args: &FleetInitArgs,
    dataset_id: &str,
    slug: &str,
    config_path: &Path,
) -> Result<String> {
    if config_path.exists() && !args.force {
        return Err(CliError::InvalidConfig(format!(
            "output {} already exists; pass --force to overwrite",
            config_path.display()
        )));
    }
    let postgres_slug = slug.replace('-', "_");
    render_init_config(&InitArgs {
        source_database_url: args.source_database_url.clone(),
        target_database_url: args.target_database_url.clone(),
        source_id: args.source_id.clone(),
        database_id: args.database_id.clone(),
        dataset_id: dataset_id.to_string(),
        publication: format!("trellara_{postgres_slug}"),
        slot: format!("trellara_{postgres_slug}_slot"),
        stream_path: args.output_dir.join("streams").join(slug),
        output: config_path.to_path_buf(),
        primary_key: args.primary_key.clone(),
        table: args.table.clone(),
        evaluate: true,
        force: args.force,
    })
}
