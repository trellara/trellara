use crate::DdlBarrierReleaseGate;

pub(crate) fn ddl_barrier_release_gates(
    release_dml: bool,
    requires_global_partition_pause: bool,
    release_blockers: &[String],
) -> Vec<DdlBarrierReleaseGate> {
    let mut gates = vec![
        gate(
            "schema_barrier_recorded",
            true,
            "barrier_lsn and schema_version are persisted before post-DDL DML can release",
        ),
        gate(
            "required_sink_acknowledgements",
            release_blockers.is_empty(),
            required_ack_evidence(release_blockers),
        ),
    ];
    if requires_global_partition_pause {
        gates.push(gate(
            "partition_visibility_watermark",
            release_blockers.is_empty(),
            "partition_visibility ACK proves every partition lane reached the barrier LSN",
        ));
    }
    gates.push(gate(
        "post_ddl_dml_release",
        release_dml,
        post_ddl_evidence(release_dml, release_blockers),
    ));
    gates
}

fn required_ack_evidence(release_blockers: &[String]) -> String {
    if release_blockers.is_empty() {
        "all required sink ACKs are accepted, match the barrier schema_version and LSN, and include detail evidence".to_string()
    } else {
        format!("blocked by {}", release_blockers.join("; "))
    }
}

fn post_ddl_evidence(release_dml: bool, release_blockers: &[String]) -> String {
    if release_dml {
        "release_dml is true with no release_blockers".to_string()
    } else {
        format!("release_dml is false: {}", release_blockers.join("; "))
    }
}

fn gate(name: &str, satisfied: bool, evidence: impl Into<String>) -> DdlBarrierReleaseGate {
    DdlBarrierReleaseGate {
        name: name.to_string(),
        satisfied,
        evidence: evidence.into(),
    }
}
