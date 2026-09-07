pub(crate) mod recommendation;
pub(crate) mod render;
pub(crate) mod tables;

use std::fs;
use std::path::Path;

use crate::{
    render_source_safety_init_config, source_safety_init_recommendation, source_safety_init_tables,
    CliError, DirectSourceSafetySummary, Result, SourceSafetyArgs, TrellaraConfig,
};

fn write_source_safety_init_config(
    args: &SourceSafetyArgs,
    tables: &[trellara_pg_capture::TablePreflight],
    target_database_url: &str,
    output: &Path,
) -> Result<()> {
    let database_url = args.database_url.as_ref().ok_or_else(|| {
        CliError::InvalidConfig("check --write-init requires --database-url".to_string())
    })?;
    let init_tables = source_safety_init_tables(args, tables)?;
    let yaml =
        render_source_safety_init_config(args, database_url, target_database_url, &init_tables)?;
    let config = TrellaraConfig::from_yaml(&yaml, &output.display().to_string())?;
    config.validate()?;
    if output.exists() && !args.force {
        return Err(CliError::InvalidConfig(format!(
            "output {} already exists; pass --force to overwrite",
            output.display()
        )));
    }
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent).map_err(|source| CliError::WriteOutput {
            path: parent.display().to_string(),
            source,
        })?;
    }
    fs::write(output, yaml).map_err(|source| CliError::WriteOutput {
        path: output.display().to_string(),
        source,
    })
}

pub(crate) fn maybe_write_source_safety_init_config(
    args: &SourceSafetyArgs,
    tables: &[trellara_pg_capture::TablePreflight],
    summary: &mut DirectSourceSafetySummary,
) -> Result<()> {
    let Some(output) = &args.write_init else {
        return Ok(());
    };
    if summary.critical_factor_count > 0 {
        return Err(CliError::InvalidConfig(
            "check --write-init refused because critical source blockers are present".to_string(),
        ));
    }
    let target_database_url = args.target_database_url.as_ref().ok_or_else(|| {
        CliError::InvalidConfig("check --write-init requires --target-database-url".to_string())
    })?;
    write_source_safety_init_config(args, tables, target_database_url, output)?;
    summary.init_config_written = Some(output.display().to_string());
    summary.init_recommendation = source_safety_init_recommendation(args, tables);
    Ok(())
}
