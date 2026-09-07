use crate::{DatasetMode, DdlPlanVerdict, DdlPropagationSink, DdlReleaseGate};

pub(crate) fn ddl_release_gates(
    dataset_mode: DatasetMode,
    verdict: DdlPlanVerdict,
    sinks: &[DdlPropagationSink],
) -> Vec<DdlReleaseGate> {
    let mut gates = Vec::new();
    gates.push(gate(
        "schema_barrier_recorded",
        "barrier_lsn, schema_version, source transaction id, and ordered DDL change set",
        "capture records the DDL transaction boundary before any later DML is published",
    ));
    gates.push(gate(
        "required_sink_acknowledgements",
        "one ack per required sink with ack_lsn, accepted schema_version, and detail evidence",
        &format!(
            "{} required sink acknowledgements match the barrier schema version and include audit detail",
            sinks.len()
        ),
    ));
    if dataset_mode == DatasetMode::PartitionedScaleMode {
        gates.push(gate(
            "partition_visibility_watermark",
            "partition-watermarks output for every partition lane at or beyond barrier_lsn",
            "global visibility advances only after all partition lanes reach the schema barrier",
        ));
    }
    gates.push(gate(
        "post_ddl_dml_release",
        "release_dml status with no release_blockers and a durable target checkpoint",
        post_ddl_release_condition(verdict, dataset_mode),
    ));
    gates
}

fn post_ddl_release_condition(verdict: DdlPlanVerdict, dataset_mode: DatasetMode) -> &'static str {
    match (verdict, dataset_mode) {
        (DdlPlanVerdict::Blocked, DatasetMode::PartitionedScaleMode) => {
            "blocked DDL is removed or reclassified before partitioned DML can leave quarantine"
        }
        (DdlPlanVerdict::Blocked, _) => {
            "blocked DDL is removed or reclassified before later DML can leave quarantine"
        }
        (_, DatasetMode::PartitionedScaleMode) => {
            "sink ACKs and partition watermarks prove post-DDL DML is visible in source order"
        }
        _ => "sink ACKs prove post-DDL DML is visible only after the DDL transaction boundary",
    }
}

fn gate(name: &str, required_evidence: &str, opens_when: &str) -> DdlReleaseGate {
    DdlReleaseGate {
        name: name.to_string(),
        required_evidence: required_evidence.to_string(),
        opens_when: opens_when.to_string(),
    }
}
