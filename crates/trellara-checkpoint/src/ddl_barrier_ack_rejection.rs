use crate::{
    parse_lsn, partition_visibility_ddl_ack_detail_is_valid, raw_cdc_lake_ddl_ack_detail_is_valid,
    spark_derived_views_ddl_ack_detail_is_valid, target_postgres_ddl_ack_detail_is_valid,
    target_postgres_ddl_ack_proof::target_postgres_ddl_ack_detail_matches_barrier, DdlBarrierAck,
};

pub(crate) struct DdlAckRejection {
    pub(crate) code: &'static str,
    pub(crate) reason: String,
}

pub(crate) fn ddl_ack_rejection(
    ack: &DdlBarrierAck,
    required_schema_version: &str,
    barrier_lsn: u64,
) -> Option<DdlAckRejection> {
    if !ack.accepted {
        Some(rejection(
            "sink_rejected_barrier",
            "sink rejected the schema barrier",
        ))
    } else if ack.schema_version != required_schema_version {
        Some(rejection(
            "schema_version_mismatch",
            format!(
                "schema_version {} does not match required {required_schema_version}",
                ack.schema_version
            ),
        ))
    } else if parse_lsn(&ack.ack_lsn) < barrier_lsn {
        Some(rejection(
            "ack_lsn_before_barrier",
            format!("ack_lsn {} is before barrier_lsn", ack.ack_lsn),
        ))
    } else if ack.sink == "target_postgres" && !target_postgres_ddl_ack_detail_is_valid(&ack.detail)
    {
        Some(rejection(
            "insufficient_target_postgres_evidence",
            "target_postgres ACK detail must include applied DDL statement evidence, plan_sha256/statement_sha256 digests, and release_gate=post_ddl_dml_release",
        ))
    } else if ack.sink == "target_postgres"
        && !target_postgres_ddl_ack_detail_matches_barrier(&ack.detail, barrier_lsn)
    {
        Some(rejection(
            "target_postgres_barrier_lsn_mismatch",
            "target_postgres ACK detail barrier_lsn must match the DDL barrier LSN",
        ))
    } else if ack.sink == "partition_visibility"
        && !partition_visibility_ddl_ack_detail_is_valid(&ack.detail, &ack.ack_lsn, barrier_lsn)
    {
        Some(rejection(
            "insufficient_partition_visibility_evidence",
            "partition_visibility ACK detail must include durable and applied watermark evidence and release_gate=post_ddl_dml_release",
        ))
    } else if ack.sink == "raw_cdc_lake"
        && !raw_cdc_lake_ddl_ack_detail_is_valid(&ack.detail, required_schema_version)
    {
        Some(rejection(
            "insufficient_raw_cdc_lake_evidence",
            "raw_cdc_lake ACK detail must include schema_version, epoch metadata, partition metadata, manifest_digest, and release_gate=post_ddl_dml_release",
        ))
    } else if ack.sink == "spark_derived_views"
        && !spark_derived_views_ddl_ack_detail_is_valid(&ack.detail)
    {
        Some(rejection(
            "insufficient_spark_derived_views_evidence",
            "spark_derived_views ACK detail must include regenerated view count, template_digest, accepted_by, and release_gate=post_ddl_dml_release",
        ))
    } else {
        None
    }
}

fn rejection(code: &'static str, reason: impl Into<String>) -> DdlAckRejection {
    DdlAckRejection {
        code,
        reason: reason.into(),
    }
}
