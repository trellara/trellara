use trellara_apply_postgres::PostgresApplyConfig;
use trellara_pg_capture::{PgCaptureConfig, TableSelector};

use crate::{Result, TableConfig, TrellaraConfig};

impl TrellaraConfig {
    pub fn to_capture_config(&self, create_if_missing: bool) -> Result<PgCaptureConfig> {
        self.validate()?;
        Ok(PgCaptureConfig {
            connection_uri: self.source.database_url.expose().to_string(),
            source_id: self.source.id.clone(),
            database_id: self
                .source
                .database_id
                .clone()
                .unwrap_or_else(|| "postgres".to_string()),
            dataset_id: self.dataset.id.clone(),
            publication_name: self.source.publication.clone(),
            slot_name: self.source.slot.clone(),
            tables: self
                .dataset
                .tables
                .iter()
                .map(|table| TableSelector::new(&table.schema, &table.name))
                .collect(),
            create_if_missing,
            stream_spill_threshold_changes: self
                .source
                .stream_spill_threshold_changes
                .unwrap_or(trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES),
            stream_spill_dir: self.source.stream_spill_dir.clone(),
            pgoutput: self.source.pgoutput.clone(),
        })
    }

    pub fn to_postgres_apply_config(
        &self,
        connection_uri: String,
        ensure_checkpoint_schema: bool,
    ) -> Result<PostgresApplyConfig> {
        self.validate()?;
        Ok(PostgresApplyConfig {
            connection_uri,
            ensure_checkpoint_schema,
            table_policies: self
                .dataset
                .tables
                .iter()
                .filter_map(TableConfig::to_apply_policy)
                .collect(),
        })
    }
}
