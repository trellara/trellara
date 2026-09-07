use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::quote_ident;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PgCaptureConfig {
    pub connection_uri: String,
    pub source_id: String,
    pub database_id: String,
    pub dataset_id: String,
    pub publication_name: String,
    pub slot_name: String,
    pub tables: Vec<TableSelector>,
    pub create_if_missing: bool,
    #[serde(default = "default_stream_spill_threshold_changes")]
    pub stream_spill_threshold_changes: usize,
    #[serde(default)]
    pub stream_spill_dir: Option<PathBuf>,
    #[serde(default)]
    pub pgoutput: PgOutputProtocolConfig,
}

pub const DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES: usize = 1_024;
pub const MAX_STREAM_SPILL_THRESHOLD_CHANGES: usize = 1_000_000;
pub const DEFAULT_PGOUTPUT_PROTOCOL_VERSION: u8 = 2;
pub const DEFAULT_PGOUTPUT_STREAMING: bool = true;

fn default_stream_spill_threshold_changes() -> usize {
    DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES
}

fn default_pgoutput_protocol_version() -> u8 {
    DEFAULT_PGOUTPUT_PROTOCOL_VERSION
}

fn default_pgoutput_streaming() -> bool {
    DEFAULT_PGOUTPUT_STREAMING
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PgOutputProtocolConfig {
    #[serde(default = "default_pgoutput_protocol_version")]
    pub protocol_version: u8,
    #[serde(default = "default_pgoutput_streaming")]
    pub streaming: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TableSelector {
    pub schema: String,
    pub name: String,
}

impl TableSelector {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
        }
    }

    pub fn to_qualified_sql(&self) -> String {
        format!("{}.{}", quote_ident(&self.schema), quote_ident(&self.name))
    }
}
