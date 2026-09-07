use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use tokio::time::sleep;
use tokio_postgres::NoTls;
use trellara_checkpoint::{FlowKey, PostgresCheckpointStore};
use trellara_stream::{PublishAck, StreamMessage, StreamPublisher};

pub(crate) type TestResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub(crate) const SOURCE_DATABASE_URL_ENV: &str = "TRELLARA_SOURCE_TEST_DATABASE_URL";
pub(crate) const SOURCE_ID: &str = "relay-source-integration";
pub(crate) const DATASET_ID: &str = "relay-sales";
pub(crate) const SLOT_NAME: &str = "trellara_relay_test_decoding_slot";
pub(crate) const PGOUTPUT_SLOT_NAME: &str = "trellara_relay_pgoutput_slot";
pub(crate) const PGOUTPUT_PUBLICATION_NAME: &str = "trellara_relay_pgoutput_publication";

#[path = "support/db.rs"]
mod db;

pub(crate) use db::*;
