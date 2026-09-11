use std::error::Error;

pub(crate) type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) const SOURCE_DATABASE_URL_ENV: &str = "TRELLARA_SOURCE_TEST_DATABASE_URL";
pub(crate) const TARGET_DATABASE_URL_ENV: &str = "TRELLARA_TARGET_TEST_DATABASE_URL";
pub(crate) const RESEED_TABLE: &str = "trellara_verify_reseed_sales";
pub(crate) const EXPORTED_RESEED_SLOT: &str = "trellara_verify_exported_reseed_slot";
pub(crate) static RESEED_INTEGRATION_LOCK: tokio::sync::Mutex<()> =
    tokio::sync::Mutex::const_new(());

#[path = "support/db.rs"]
mod db;

pub(crate) use db::*;
