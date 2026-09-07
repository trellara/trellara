use crate::{CaptureError, Result};

use super::config::{
    PgCaptureConfig, PgOutputProtocolConfig, DEFAULT_PGOUTPUT_PROTOCOL_VERSION,
    DEFAULT_PGOUTPUT_STREAMING, MAX_STREAM_SPILL_THRESHOLD_CHANGES,
};

impl PgCaptureConfig {
    pub fn validate(&self) -> Result<()> {
        validate_non_empty("connection_uri", &self.connection_uri)?;
        validate_non_empty("source_id", &self.source_id)?;
        validate_non_empty("dataset_id", &self.dataset_id)?;
        validate_non_empty("publication_name", &self.publication_name)?;
        validate_non_empty("slot_name", &self.slot_name)?;

        if self.tables.is_empty() {
            return invalid_config("at least one table selector is required");
        }
        if self.stream_spill_threshold_changes == 0 {
            return invalid_config("stream_spill_threshold_changes must be greater than zero");
        }
        if self.stream_spill_threshold_changes > MAX_STREAM_SPILL_THRESHOLD_CHANGES {
            return invalid_config(format!(
                "stream_spill_threshold_changes must be at most {MAX_STREAM_SPILL_THRESHOLD_CHANGES}"
            ));
        }
        if let Some(stream_spill_dir) = &self.stream_spill_dir {
            if stream_spill_dir.as_os_str().is_empty() {
                return invalid_config("stream_spill_dir must not be empty");
            }
        }

        self.pgoutput.validate()
    }
}

impl Default for PgOutputProtocolConfig {
    fn default() -> Self {
        Self {
            protocol_version: DEFAULT_PGOUTPUT_PROTOCOL_VERSION,
            streaming: DEFAULT_PGOUTPUT_STREAMING,
        }
    }
}

impl PgOutputProtocolConfig {
    pub fn validate(&self) -> Result<()> {
        if !matches!(self.protocol_version, 1 | 2) {
            return Err(CaptureError::InvalidConfig(format!(
                "pgoutput.protocol_version must be 1 or 2, got {}",
                self.protocol_version
            )));
        }
        if self.streaming && self.protocol_version < 2 {
            return invalid_config("pgoutput.streaming requires protocol_version 2");
        }
        Ok(())
    }

    pub(crate) fn start_options<'a>(
        &'a self,
        publication_name: &'a str,
        protocol_version: &'a str,
        streaming: &'a str,
    ) -> Vec<(&'static str, &'a str)> {
        let mut options = vec![
            ("proto_version", protocol_version),
            ("publication_names", publication_name),
        ];
        if self.protocol_version >= 2 {
            options.push(("streaming", streaming));
        }
        options
    }
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return invalid_config(format!("{field_name} must not be empty"));
    }
    Ok(())
}

fn invalid_config<T>(message: impl Into<String>) -> Result<T> {
    Err(CaptureError::InvalidConfig(message.into()))
}
