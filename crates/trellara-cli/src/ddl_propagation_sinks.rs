use crate::{DdlPropagationSink, DdlPropagationSinkKind, TrellaraConfig};

pub(crate) fn ddl_propagation_sinks(
    config: &TrellaraConfig,
    requires_global_partition_pause: bool,
) -> Vec<DdlPropagationSink> {
    let mut sinks = Vec::new();
    if config.target.is_some() {
        sinks.push(DdlPropagationSink {
            name: "target_postgres".to_string(),
            kind: DdlPropagationSinkKind::TargetPostgres,
            required_ack:
                "target schema fingerprint matches accepted source schema version at barrier LSN"
                    .to_string(),
            ack_evidence:
                "target preflight JSON with schema_version, schema_fingerprint, ack_lsn, plan_sha256, and statement_sha256 values"
                    .to_string(),
        });
    }
    sinks.push(DdlPropagationSink {
        name: "raw_cdc_lake".to_string(),
        kind: DdlPropagationSinkKind::RawCdcLake,
        required_ack:
            "raw CDC table metadata records the new schema version before epoch metadata is published"
                .to_string(),
        ack_evidence: "lake writer plan or epoch metadata row carrying schema_version and barrier_lsn"
            .to_string(),
    });
    sinks.push(DdlPropagationSink {
        name: "spark_derived_views".to_string(),
        kind: DdlPropagationSinkKind::SparkDerivedView,
        required_ack:
            "current-state and SCD2 templates are regenerated or explicitly accepted for the schema version"
                .to_string(),
        ack_evidence:
            "rendered Spark template_sha256 digest plus operator acceptance for the schema version"
                .to_string(),
    });
    if requires_global_partition_pause {
        sinks.push(DdlPropagationSink {
            name: "partition_visibility".to_string(),
            kind: DdlPropagationSinkKind::PartitionVisibility,
            required_ack:
                "every partition lane has reached the schema barrier before global visibility advances"
                    .to_string(),
            ack_evidence:
                "partition-watermarks output showing every partition at or beyond barrier_lsn"
                    .to_string(),
        });
    }
    sinks
}
