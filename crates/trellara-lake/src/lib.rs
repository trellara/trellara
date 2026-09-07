mod commit_manifest_orders;
mod commit_manifest_validation;
mod commit_materialization_current;
mod commit_materialization_operation;
mod commit_materialization_raw;
mod commit_materialization_scd2;
mod commit_operation;
mod commit_plan;
mod commit_row;
mod commit_visibility;
mod config;
mod consumer_gate;
mod consumer_gate_completeness;
mod consumer_gate_consistency;
mod ddl_ack;
mod ddl_ack_validation;
mod derived_views_ddl_ack;
mod epoch;
mod epoch_accumulator;
mod epoch_completeness;
mod epoch_identity;
mod epoch_lsn_window;
mod epoch_recovery;
mod epoch_replay_ledger;
mod epoch_summary;
mod epoch_summary_accumulate;
mod epoch_summary_builder;
mod error;
mod raw_cdc;
mod raw_cdc_accumulator;
mod raw_cdc_metadata_types;
mod raw_cdc_naming;
mod raw_cdc_row_intent_types;
mod raw_cdc_types;
mod types;

pub(crate) use commit_manifest_validation::validate_epoch_manifest;
pub use commit_plan::plan_commit;
pub use config::{LakePlanConfig, LakeTableConfig};
pub use consumer_gate::{
    ensure_lake_epoch_consumable, LakeEpochConsumerDecision, LakeEpochConsumerOptions,
    LAKE_EPOCH_CONSUMER_GATE_CONTRACT,
};
pub use ddl_ack::{
    raw_cdc_lake_ddl_ack_evidence, RawCdcLakeDdlAckEvidence, RawCdcLakeDdlAckRequest,
};
pub use derived_views_ddl_ack::{
    spark_derived_views_ddl_ack_evidence, SparkDerivedViewsDdlAckEvidence,
    SparkDerivedViewsDdlAckRequest,
};
pub use epoch::{
    LakeCompletenessState, LakeEpoch, LakeEpochConfig, LakeEpochPartition, LakeEpochSource,
    LakeEpochSourceState, LakeEpochTable, LakeEpochVerification, LakeEpochVerificationStatus,
    LakeStragglerPolicy,
};
pub use epoch_identity::{deterministic_epoch_id, LakeEpochSourceWindow};
pub use epoch_recovery::{lake_epoch_recovery_guidance, LakeEpochRecoveryGuidance};
pub use epoch_summary::build_epoch_summary;
pub use error::LakeError;
pub use raw_cdc::{plan_raw_cdc_epoch_writes, raw_cdc_epoch_metadata_object_key_hint};
#[cfg(feature = "parquet-writer")]
pub use raw_cdc::{
    raw_cdc_epoch_metadata_parquet_arrow_schema, raw_cdc_parquet_arrow_schema,
    write_raw_cdc_epoch_metadata_parquet_file, write_raw_cdc_parquet_file,
    LakeRawCdcEpochMetadataParquetWriteOutput, LakeRawCdcParquetWriteOutput,
    RAW_CDC_PARQUET_FIELD_ID_KEY,
};
pub use raw_cdc_types::{
    LakeRawCdcCommitStep, LakeRawCdcCommitterTopology, LakeRawCdcDataFilePlan,
    LakeRawCdcDuplicateReplayEvidence, LakeRawCdcEpochMetadataParquetWriteEvidence,
    LakeRawCdcEpochMetadataPlan, LakeRawCdcEpochPartitionRow, LakeRawCdcEpochQuarantineRow,
    LakeRawCdcEpochRow, LakeRawCdcEpochSourceRow, LakeRawCdcEpochTableRow,
    LakeRawCdcEpochVerificationRow, LakeRawCdcEpochWritePlan, LakeRawCdcParquetWriteEvidence,
    LakeRawCdcRecoveryScenario, LakeRawCdcRowIntent, LakeRawCdcTableCommitter,
    LakeRawCdcWriterConfig, RAW_CDC_SOURCE_ACK_BOUNDARY,
};
pub use types::{
    LakeColumnValue, LakeCommitPlan, LakeMaterialization, LakeValue, LakeVisibilityBoundary,
    LakeWriteKind, LakeWriteOperation,
};

pub fn raw_cdc_epoch_verification_id(
    dataset_id: &str,
    epoch_id: &str,
    checksum_rollup: u64,
) -> String {
    raw_cdc::verification_id(dataset_id, epoch_id, checksum_rollup)
}

#[cfg(test)]
mod tests;
