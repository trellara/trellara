use crate::{
    require_non_empty, CliError, ConfigurationEnvironment, DatasetMode, KafkaProfile, Result,
    SourceCaptureKind, StreamConfig, TrellaraConfig,
};

impl TrellaraConfig {
    pub fn validate(&self) -> Result<()> {
        self.validate_environment_schema()?;
        require_non_empty("source.id", &self.source.id)?;
        require_non_empty("source.database_url", self.source.database_url.expose())?;
        require_non_empty("source.publication", &self.source.publication)?;
        require_non_empty("source.slot", &self.source.slot)?;
        if self
            .source
            .wal_retention_warn_bytes
            .is_some_and(|bytes| bytes <= 0)
        {
            return Err(CliError::InvalidConfig(
                "source.wal_retention_warn_bytes must be greater than zero".to_string(),
            ));
        }
        if self.source.stream_spill_threshold_changes == Some(0) {
            return Err(CliError::InvalidConfig(
                "source.stream_spill_threshold_changes must be greater than zero".to_string(),
            ));
        }
        if self
            .source
            .stream_spill_threshold_changes
            .is_some_and(|threshold| {
                threshold > trellara_pg_capture::MAX_STREAM_SPILL_THRESHOLD_CHANGES
            })
        {
            return Err(CliError::InvalidConfig(format!(
                "source.stream_spill_threshold_changes must be at most {}",
                trellara_pg_capture::MAX_STREAM_SPILL_THRESHOLD_CHANGES
            )));
        }
        if let Some(stream_spill_dir) = &self.source.stream_spill_dir {
            if stream_spill_dir.as_os_str().is_empty() {
                return Err(CliError::InvalidConfig(
                    "source.stream_spill_dir must not be empty".to_string(),
                ));
            }
        }
        if self.source.capture == SourceCaptureKind::PgOutput {
            self.source
                .pgoutput
                .validate()
                .map_err(|source| CliError::InvalidConfig(format!("source {}", source)))?;
        }
        require_non_empty("dataset.id", &self.dataset.id)?;
        if self.dataset.tables.is_empty() {
            return Err(CliError::InvalidConfig(
                "dataset.tables must include at least one table".to_string(),
            ));
        }
        for table in &self.dataset.tables {
            require_non_empty("dataset.tables[].schema", &table.schema)?;
            require_non_empty("dataset.tables[].name", &table.name)?;
            if let Some(verify) = &table.verify {
                require_non_empty("dataset.tables[].verify.primary_key", &verify.primary_key)?;
                if let Some(row_filter) = &verify.row_filter {
                    require_non_empty("dataset.tables[].verify.row_filter", row_filter)?;
                }
            }
            if let Some(contract) = &table.contract {
                if contract.source_schema_fingerprint == Some(0) {
                    return Err(CliError::InvalidConfig(
                        "dataset.tables[].contract.source_schema_fingerprint must be greater than zero"
                            .to_string(),
                    ));
                }
                for column in &contract.target_owned_columns {
                    require_non_empty("dataset.tables[].contract.target_owned_columns[]", column)?;
                }
            }
        }
        self.dataset
            .mode
            .validate(self.dataset.partition.as_ref())?;
        if let Some(strict_chunking) = &self.dataset.strict_chunking {
            if self.dataset.mode != DatasetMode::StrictTransactionOrder {
                return Err(CliError::InvalidConfig(
                    "dataset.strict_chunking is only supported for strict_transaction_order"
                        .to_string(),
                ));
            }
            if strict_chunking.max_changes_per_chunk == 0 {
                return Err(CliError::InvalidConfig(
                    "dataset.strict_chunking.max_changes_per_chunk must be greater than zero"
                        .to_string(),
                ));
            }
        }
        self.stream.validate()?;
        if let Some(target) = &self.target {
            require_non_empty("target.database_url", target.database_url.expose())?;
        }
        Ok(())
    }

    fn validate_environment_schema(&self) -> Result<()> {
        let kafka_profile = match &self.stream {
            StreamConfig::Kafka { profile, .. } => Some(profile),
            StreamConfig::Local { .. } => None,
        };
        match self.environment {
            ConfigurationEnvironment::Development => {
                if matches!(kafka_profile, Some(KafkaProfile::Production { .. })) {
                    return Err(CliError::InvalidConfig(
                        "environment=development requires stream.profile.kind=development"
                            .to_string(),
                    ));
                }
            }
            ConfigurationEnvironment::Production => {
                if !self.source.database_url.is_reference()
                    || self
                        .target
                        .as_ref()
                        .is_some_and(|target| !target.database_url.is_reference())
                {
                    return Err(CliError::InvalidConfig(
                        "production database_url fields must use environment_variable or file secret references"
                            .to_string(),
                    ));
                }
                if !matches!(kafka_profile, Some(KafkaProfile::Production { .. })) {
                    return Err(CliError::InvalidConfig(
                        "environment=production requires Kafka stream.profile.kind=production"
                            .to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}
