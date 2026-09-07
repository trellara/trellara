use trellara_pg_capture::PgCaptureConfig;

use crate::{
    parse_init_table, reject_surrounding_whitespace, require_non_empty, CliError, Result,
    SourceCaptureKind, SourceSafetyArgs,
};

pub(crate) struct DirectSourceSafetyCaptureConfig {
    pub(crate) capture_kind: SourceCaptureKind,
    pub(crate) capture_config: PgCaptureConfig,
}

pub(crate) fn direct_source_safety_capture_config(
    args: &SourceSafetyArgs,
) -> Result<DirectSourceSafetyCaptureConfig> {
    let database_url = args.database_url.as_ref().ok_or_else(|| {
        CliError::InvalidConfig("check requires either --config or --database-url".to_string())
    })?;
    require_clean_direct_value("database_url", database_url)?;
    require_clean_direct_value("source_id", &args.source_id)?;
    require_clean_direct_value("database_id", &args.database_id)?;
    require_clean_direct_value("dataset_id", &args.dataset_id)?;
    require_clean_direct_value("publication", &args.publication)?;
    require_clean_direct_value("slot", &args.slot)?;
    require_clean_direct_value("capture", &args.capture)?;
    if args.write_init.is_some() {
        let target_database_url = args.target_database_url.as_ref().ok_or_else(|| {
            CliError::InvalidConfig("check --write-init requires --target-database-url".to_string())
        })?;
        require_clean_direct_value("target_database_url", target_database_url)?;
    }
    validate_wal_retention_threshold(args.wal_retention_warn_bytes)?;

    if args.table.is_empty() {
        return Err(CliError::InvalidConfig(
            "check --database-url requires at least one --table schema.table".to_string(),
        ));
    }

    let capture_kind = parse_source_capture_kind(&args.capture)?;
    let tables = args
        .table
        .iter()
        .map(|table| {
            require_clean_direct_value("table", table)?;
            parse_init_table(table)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(DirectSourceSafetyCaptureConfig {
        capture_kind,
        capture_config: PgCaptureConfig {
            connection_uri: database_url.clone(),
            source_id: args.source_id.clone(),
            database_id: args.database_id.clone(),
            dataset_id: args.dataset_id.clone(),
            publication_name: args.publication.clone(),
            slot_name: args.slot.clone(),
            tables,
            create_if_missing: false,
            stream_spill_threshold_changes:
                trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
            stream_spill_dir: None,
            pgoutput: Default::default(),
        },
    })
}

fn require_clean_direct_value(field: &'static str, value: &str) -> Result<()> {
    require_non_empty(field, value)?;
    reject_surrounding_whitespace(field, value)
}

fn validate_wal_retention_threshold(threshold: Option<i64>) -> Result<()> {
    if threshold.is_some_and(|threshold| threshold <= 0) {
        return Err(CliError::InvalidConfig(
            "wal_retention_warn_bytes must be greater than zero".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn parse_source_capture_kind(value: &str) -> Result<SourceCaptureKind> {
    match value {
        "pgoutput" => Ok(SourceCaptureKind::PgOutput),
        "test_decoding" => Ok(SourceCaptureKind::TestDecoding),
        _ => Err(CliError::InvalidConfig(format!(
            "capture must be pgoutput or test_decoding, got {value:?}"
        ))),
    }
}
