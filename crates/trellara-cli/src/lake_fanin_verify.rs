use crate::{LakeFaninVerifyMismatch, LakeFaninVerifyMismatchSeverity, LakeFaninVerifyStatus};

pub(crate) fn push_lake_fanin_mismatch(
    mismatches: &mut Vec<LakeFaninVerifyMismatch>,
    field: &str,
    stream_value: &str,
    lake_value: &str,
    severity: LakeFaninVerifyMismatchSeverity,
) {
    if stream_value != lake_value {
        mismatches.push(LakeFaninVerifyMismatch {
            field: field.to_string(),
            stream_value: stream_value.to_string(),
            lake_value: lake_value.to_string(),
            severity,
        });
    }
}

pub(crate) fn lake_fanin_verify_recommended_next_steps(
    status: LakeFaninVerifyStatus,
    spark_consumption_allowed: bool,
) -> Vec<String> {
    match (status, spark_consumption_allowed) {
        (LakeFaninVerifyStatus::Match, true) => vec![
            "publish the lake verification artifact with the pilot package".to_string(),
            "run Spark templates for consumers that accept this epoch state".to_string(),
        ],
        (LakeFaninVerifyStatus::Match, false) => vec![
            "keep the epoch out of Spark-derived current-state and SCD2 outputs".to_string(),
            "resolve lake epoch state or verification_status before consumer visibility"
                .to_string(),
        ],
        (LakeFaninVerifyStatus::Mismatch, _) => vec![
            "review warning mismatches before publishing the epoch".to_string(),
            "rerun the lake fan-in writer or refresh the epoch proof artifacts".to_string(),
        ],
        (LakeFaninVerifyStatus::Blocked, _) => vec![
            "do not expose this epoch to Spark consumers".to_string(),
            "compare _trellara_epochs and _trellara_epoch_sources against stream evidence"
                .to_string(),
            "rerun trellara lake fanin verify after repair or replay".to_string(),
        ],
    }
}
