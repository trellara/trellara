use std::error::Error;

use trellara_checkpoint::{FlowKey, PostgresCheckpointStore};

pub(crate) type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) const TEST_DATABASE_URL_ENV: &str = "TRELLARA_TEST_DATABASE_URL";
pub(crate) const SOURCE_ID: &str = "checkpoint-source";
pub(crate) const DATABASE_ID: &str = "checkpoint-database";
pub(crate) const DATASET_ID: &str = "checkpoint-dataset";

#[path = "support/db.rs"]
mod db;

pub(crate) use db::*;

pub(crate) async fn connect_store(database_url: &str) -> TestResult<PostgresCheckpointStore> {
    Ok(PostgresCheckpointStore::connect(database_url, true).await?)
}

pub(crate) fn flow_key() -> FlowKey {
    FlowKey::new(SOURCE_ID, DATASET_ID)
}
