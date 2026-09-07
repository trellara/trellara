use std::error::Error;

pub(crate) type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

const TEST_DATABASE_URL_ENV: &str = "TRELLARA_TEST_DATABASE_URL";
pub(crate) const DEFAULT_TEST_DATABASE_URL: &str =
    "postgresql://trellara:trellara@localhost:55433/trellara_target";
const SOURCE_ID: &str = "source-integration";
const DATASET_ID: &str = "retail.integration";
const DATABASE_ID: &str = "target-postgres";
pub(crate) const TABLE_NAME: &str = "trellara_apply_sales";
pub(crate) static INTEGRATION_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

#[path = "support/db.rs"]
mod db;
#[path = "support/envelopes.rs"]
mod envelopes;

pub(crate) use db::*;
pub(crate) use envelopes::*;
