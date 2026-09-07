use trellara_protocol::TransactionEnvelope;

use crate::raw_cdc_types::{LakeRawCdcEpochWritePlan, LakeRawCdcWriterConfig};
use crate::{LakeError, LakePlanConfig};

mod commit_steps;
mod committer;
mod ddl_metadata;
mod ddl_metadata_validation;
mod dedup;
mod dedup_key;
#[cfg(test)]
#[path = "tests/tests_raw_cdc_dedup.rs"]
mod dedup_tests;
mod file_change;
mod manifest;
mod materialize;
pub(crate) mod metadata;
#[cfg(feature = "parquet-writer")]
mod parquet_writer;
mod planner;
mod planner_accumulate;
mod planner_admission;
mod recovery;
mod row_intent;
mod row_manifest;

pub(crate) use manifest::verification_id;
#[cfg(feature = "parquet-writer")]
pub use parquet_writer::{
    raw_cdc_epoch_metadata_parquet_arrow_schema, raw_cdc_parquet_arrow_schema,
    write_raw_cdc_epoch_metadata_parquet_file, write_raw_cdc_parquet_file,
    LakeRawCdcEpochMetadataParquetWriteOutput, LakeRawCdcParquetWriteOutput,
    RAW_CDC_PARQUET_FIELD_ID_KEY,
};
use planner::RawCdcEpochPlanner;

#[must_use]
pub fn raw_cdc_epoch_metadata_object_key_hint(epoch_id: &str, table_name: &str) -> String {
    crate::raw_cdc_naming::epoch_metadata_object_key_hint(epoch_id, table_name)
}

pub fn plan_raw_cdc_epoch_writes(
    writer_config: &LakeRawCdcWriterConfig,
    plan_config: &LakePlanConfig,
    envelopes: &[TransactionEnvelope],
) -> Result<LakeRawCdcEpochWritePlan, LakeError> {
    RawCdcEpochPlanner::new(writer_config, plan_config)?.plan(envelopes)
}
