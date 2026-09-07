use std::path::PathBuf;

use serde::Deserialize;
use trellara_pg_capture::PgOutputProtocolConfig;

use crate::SensitiveString;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SourceConfig {
    pub id: String,
    pub database_url: SensitiveString,
    #[serde(default)]
    pub database_id: Option<String>,
    #[serde(default)]
    pub capture: SourceCaptureKind,
    #[serde(default)]
    pub wal_retention_warn_bytes: Option<i64>,
    #[serde(default)]
    pub stream_spill_threshold_changes: Option<usize>,
    #[serde(default)]
    pub stream_spill_dir: Option<PathBuf>,
    #[serde(default)]
    pub pgoutput: PgOutputProtocolConfig,
    pub publication: String,
    pub slot: String,
}

#[derive(Copy, Clone, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceCaptureKind {
    #[default]
    PgOutput,
    TestDecoding,
}

impl SourceCaptureKind {
    pub(crate) fn expected_plugin(&self) -> &'static str {
        match self {
            SourceCaptureKind::PgOutput => "pgoutput",
            SourceCaptureKind::TestDecoding => "test_decoding",
        }
    }
}
