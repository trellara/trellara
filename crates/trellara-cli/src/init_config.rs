use trellara_pg_capture::TableSelector;

use crate::{require_non_empty, yaml_string, CliError, InitArgs, Result};

pub(crate) fn render_init_config(args: &InitArgs) -> Result<String> {
    require_non_empty("source_database_url", &args.source_database_url)?;
    require_non_empty("source_id", &args.source_id)?;
    require_non_empty("database_id", &args.database_id)?;
    require_non_empty("dataset_id", &args.dataset_id)?;
    require_non_empty("publication", &args.publication)?;
    require_non_empty("slot", &args.slot)?;
    require_non_empty("primary_key", &args.primary_key)?;
    if args.table.is_empty() {
        return Err(CliError::InvalidConfig(
            "init requires at least one --table schema.table".to_string(),
        ));
    }

    let tables = args
        .table
        .iter()
        .map(|table| parse_init_table(table))
        .collect::<Result<Vec<_>>>()?;

    let mut yaml = String::new();
    yaml.push_str("config_version: 2\n");
    yaml.push_str("environment: development\n\n");
    yaml.push_str("source:\n");
    yaml.push_str(&format!("  id: {}\n", yaml_string(&args.source_id)));
    yaml.push_str(&format!(
        "  database_url: {}\n",
        yaml_string(&args.source_database_url)
    ));
    yaml.push_str(&format!(
        "  database_id: {}\n",
        yaml_string(&args.database_id)
    ));
    yaml.push_str(&format!(
        "  publication: {}\n",
        yaml_string(&args.publication)
    ));
    yaml.push_str(&format!("  slot: {}\n", yaml_string(&args.slot)));
    yaml.push_str(&format!(
        "  stream_spill_threshold_changes: {}\n",
        trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES
    ));
    yaml.push_str("  stream_spill_dir: './target/trellara-spill'\n\n");
    yaml.push_str("dataset:\n");
    yaml.push_str(&format!("  id: {}\n", yaml_string(&args.dataset_id)));
    yaml.push_str("  mode: strict_transaction_order\n");
    yaml.push_str("  strict_chunking:\n");
    yaml.push_str("    max_changes_per_chunk: 1000\n");
    yaml.push_str("  tables:\n");
    for table in &tables {
        yaml.push_str(&format!("    - schema: {}\n", yaml_string(&table.schema)));
        yaml.push_str(&format!("      name: {}\n", yaml_string(&table.name)));
        yaml.push_str("      verify:\n");
        yaml.push_str(&format!(
            "        primary_key: {}\n",
            yaml_string(&args.primary_key)
        ));
    }
    yaml.push_str("\nstream:\n");
    yaml.push_str("  kind: local\n");
    yaml.push_str(&format!(
        "  path: {}\n",
        yaml_string(&args.stream_path.display().to_string())
    ));
    yaml.push_str("  durability: fsync\n");
    if let Some(target_database_url) = &args.target_database_url {
        require_non_empty("target_database_url", target_database_url)?;
        yaml.push_str("\ntarget:\n");
        yaml.push_str(&format!(
            "  database_url: {}\n",
            yaml_string(target_database_url)
        ));
    }

    Ok(yaml)
}

pub(crate) fn parse_init_table(table: &str) -> Result<TableSelector> {
    let (schema, name) = table.split_once('.').ok_or_else(|| {
        CliError::InvalidConfig(format!("table {table:?} must use schema.table format"))
    })?;
    require_non_empty("table schema", schema)?;
    require_non_empty("table name", name)?;
    crate::reject_surrounding_whitespace("table schema", schema)?;
    crate::reject_surrounding_whitespace("table name", name)?;
    if name.contains('.') {
        return Err(CliError::InvalidConfig(format!(
            "table {table:?} must use schema.table format"
        )));
    }
    Ok(TableSelector::new(schema, name))
}
