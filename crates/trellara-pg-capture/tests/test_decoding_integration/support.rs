use std::error::Error;

pub(crate) type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) const TEST_DATABASE_URL_ENV: &str = "TRELLARA_SOURCE_TEST_DATABASE_URL";
pub(crate) const DEFAULT_DATABASE_URL: &str =
    "postgresql://trellara:trellara@localhost:55432/trellara_source";
pub(crate) const SLOT_NAME: &str = "trellara_test_decoding_integration_slot";
pub(crate) const TRUNCATE_SLOT_NAME: &str = "trellara_test_decoding_truncate_slot";
pub(crate) const TRUNCATE_TABLE_NAME: &str = "trellara_capture_truncate";
pub(crate) const PGOUTPUT_SLOT_NAME: &str = "trellara_pgoutput_bootstrap_slot";
pub(crate) const PGOUTPUT_PUBLICATION_NAME: &str = "trellara_pgoutput_bootstrap_publication";
pub(crate) const EXPORTED_SNAPSHOT_SLOT_NAME: &str = "trellara_exported_snapshot_slot";
pub(crate) const PGOUTPUT_SQL_SLOT_NAME: &str = "trellara_pgoutput_sql_decode_slot";
pub(crate) const PGOUTPUT_SQL_PUBLICATION_NAME: &str = "trellara_pgoutput_sql_decode_publication";
pub(crate) const PGOUTPUT_STREAM_SLOT_NAME: &str = "trellara_pgoutput_stream_slot";
pub(crate) const PGOUTPUT_STREAM_PUBLICATION_NAME: &str = "trellara_pgoutput_stream_publication";

#[path = "support/db.rs"]
mod db;

pub(crate) use db::*;
