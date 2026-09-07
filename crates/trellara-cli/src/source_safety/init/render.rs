use crate::{
    require_non_empty, yaml_string, CliError, Result, SourceSafetyArgs, SourceSafetyInitTable,
};

pub(crate) fn render_source_safety_init_config(
    args: &SourceSafetyArgs,
    source_database_url: &str,
    target_database_url: &str,
    tables: &[SourceSafetyInitTable],
) -> Result<String> {
    require_non_empty("source_database_url", source_database_url)?;
    require_non_empty("target_database_url", target_database_url)?;
    require_non_empty("source_id", &args.source_id)?;
    require_non_empty("database_id", &args.database_id)?;
    require_non_empty("dataset_id", &args.dataset_id)?;
    require_non_empty("publication", &args.publication)?;
    require_non_empty("slot", &args.slot)?;
    if tables.is_empty() {
        return Err(CliError::InvalidConfig(
            "check --write-init requires at least one inspected or configured table".to_string(),
        ));
    }

    let mut yaml = String::new();
    yaml.push_str("source:\n");
    yaml.push_str(&format!("  id: {}\n", yaml_string(&args.source_id)));
    yaml.push_str(&format!(
        "  database_url: {}\n",
        yaml_string(source_database_url)
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
    for table in tables {
        require_non_empty("dataset.tables[].verify.primary_key", &table.primary_key)?;
        yaml.push_str(&format!(
            "    - schema: {}\n",
            yaml_string(&table.selector.schema)
        ));
        yaml.push_str(&format!(
            "      name: {}\n",
            yaml_string(&table.selector.name)
        ));
        yaml.push_str("      verify:\n");
        yaml.push_str(&format!(
            "        primary_key: {}\n",
            yaml_string(&table.primary_key)
        ));
        if let Some(fingerprint) = table.source_schema_fingerprint {
            yaml.push_str("      contract:\n");
            yaml.push_str(&format!(
                "        source_schema_fingerprint: {fingerprint}\n"
            ));
        }
    }
    yaml.push_str("\nstream:\n");
    yaml.push_str("  kind: local\n");
    yaml.push_str("  path: './target/trellara-local-stream'\n");
    yaml.push_str("  durability: fsync\n");
    yaml.push_str("\ntarget:\n");
    yaml.push_str(&format!(
        "  database_url: {}\n",
        yaml_string(target_database_url)
    ));

    Ok(yaml)
}
