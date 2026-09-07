use serde_json::Value;
use trellara_checkpoint::{
    parse_lsn, partition_visibility_ddl_ack_detail_is_valid, raw_cdc_lake_ddl_ack_detail_is_valid,
    spark_derived_views_ddl_ack_detail_is_valid, target_postgres_ddl_ack_detail_is_valid,
    target_postgres_ddl_ack_detail_matches_barrier,
};

use super::sinks::sink_is_supported;

pub(super) fn is_valid(sink: &str, item: &Value, ack_lsn: &str, proof: &Value) -> bool {
    match sink {
        "partition_visibility" => partition_visibility_evidence_is_valid(item, ack_lsn, proof),
        "raw_cdc_lake" => raw_cdc_lake_evidence_is_valid(item, proof),
        "spark_derived_views" => spark_derived_views_evidence_is_valid(item),
        "target_postgres" => target_postgres_evidence_is_valid(item, proof),
        _ => sink_is_supported(sink),
    }
}

fn target_postgres_evidence_is_valid(item: &Value, proof: &Value) -> bool {
    let Some(detail) = item.get("detail").and_then(Value::as_str) else {
        return false;
    };
    let Some(barrier_lsn) = proof.get("barrier_lsn").and_then(Value::as_str) else {
        return false;
    };

    target_postgres_ddl_ack_detail_is_valid(detail)
        && super::evidence::lsn_is_valid(barrier_lsn)
        && target_postgres_ddl_ack_detail_matches_barrier(detail, parse_lsn(barrier_lsn))
}

fn raw_cdc_lake_evidence_is_valid(item: &Value, proof: &Value) -> bool {
    let Some(detail) = item.get("detail").and_then(Value::as_str) else {
        return false;
    };
    let Some(schema_version) = proof.get("schema_version").and_then(Value::as_str) else {
        return false;
    };

    raw_cdc_lake_ddl_ack_detail_is_valid(detail, schema_version)
}

fn spark_derived_views_evidence_is_valid(item: &Value) -> bool {
    item.get("detail")
        .and_then(Value::as_str)
        .is_some_and(spark_derived_views_ddl_ack_detail_is_valid)
}

fn partition_visibility_evidence_is_valid(item: &Value, ack_lsn: &str, proof: &Value) -> bool {
    let Some(detail) = item.get("detail").and_then(Value::as_str) else {
        return false;
    };
    let Some(barrier_lsn) = proof.get("barrier_lsn").and_then(Value::as_str) else {
        return false;
    };

    super::evidence::lsn_is_valid(barrier_lsn)
        && partition_visibility_ddl_ack_detail_is_valid(detail, ack_lsn, parse_lsn(barrier_lsn))
}
