use serde::Serialize;

use crate::{
    materialize_current_state, materialize_scd2, pilot_package_spark_golden_rows::raw_cdc_rows,
    TrellaraConfig,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SparkGoldenFixture {
    pub(crate) description: String,
    pub(crate) epoch_id: String,
    pub(crate) primary_key_column: String,
    pub(crate) epoch: SparkGoldenEpoch,
    pub(crate) verification: SparkGoldenVerification,
    pub(crate) raw_cdc: Vec<SparkGoldenRawCdcRow>,
    pub(crate) expected_current_state: Vec<SparkGoldenCurrentStateRow>,
    pub(crate) expected_scd2: Vec<SparkGoldenScd2Row>,
    pub(crate) rerun_idempotency_rule: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SparkGoldenEpoch {
    pub(crate) epoch_id: String,
    pub(crate) dataset_id: String,
    pub(crate) state: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SparkGoldenVerification {
    pub(crate) epoch_id: String,
    pub(crate) checksum_status: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SparkGoldenRawCdcRow {
    pub(crate) epoch_id: String,
    pub(crate) source_id: String,
    pub(crate) relation: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) commit_timestamp: String,
    pub(crate) total_order: u64,
    pub(crate) operation: String,
    pub(crate) idempotency_key: String,
    #[serde(rename = "__trellara_payload_before")]
    pub(crate) payload_before: Option<String>,
    #[serde(rename = "__trellara_payload_after")]
    pub(crate) payload_after: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SparkGoldenCurrentStateRow {
    pub(crate) source_id: String,
    pub(crate) relation: String,
    pub(crate) record_key: String,
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) idempotency_key: String,
    pub(crate) row_after_json: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SparkGoldenScd2Row {
    pub(crate) source_id: String,
    pub(crate) relation: String,
    pub(crate) record_key: String,
    pub(crate) valid_from: String,
    pub(crate) valid_to: Option<String>,
    pub(crate) is_current: bool,
    pub(crate) open_idempotency_key: String,
    pub(crate) close_commit_lsn: Option<String>,
    pub(crate) row_after_json: String,
}

impl SparkGoldenFixture {
    pub(crate) fn from_config(config: &TrellaraConfig) -> Self {
        let epoch_id = "epoch-2026-08-16T00".to_string();
        let raw_cdc = raw_cdc_rows(&epoch_id);
        Self {
            description:
                "deterministic raw CDC fixture proving Spark current-state and SCD2 derivations"
                    .to_string(),
            epoch_id: epoch_id.clone(),
            primary_key_column: "id".to_string(),
            epoch: SparkGoldenEpoch {
                epoch_id: epoch_id.clone(),
                dataset_id: config.dataset.id.clone(),
                state: "complete".to_string(),
            },
            verification: SparkGoldenVerification {
                epoch_id,
                checksum_status: "match".to_string(),
            },
            expected_current_state: materialize_current_state(&raw_cdc),
            expected_scd2: materialize_scd2(&raw_cdc),
            raw_cdc,
            rerun_idempotency_rule:
                "rerunning the same epoch must produce the same current-state and SCD2 rows because __trellara_idempotency_key is stable per source transaction change"
                    .to_string(),
        }
    }
}
