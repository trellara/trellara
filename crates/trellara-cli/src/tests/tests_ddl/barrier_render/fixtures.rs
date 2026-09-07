use super::*;

pub(super) fn ddl_rejected_sink_evidence(sink: &str, code: &str) -> DdlBarrierSinkEvidence {
    DdlBarrierSinkEvidence {
        sink: sink.to_string(),
        status: "rejected".to_string(),
        ack_lsn: Some("0/16B9000".to_string()),
        schema_version: Some("schema-v1".to_string()),
        accepted: Some(true),
        detail: Some("schema mismatch".to_string()),
        release_eligible: false,
        rejection_code: Some(code.to_string()),
        rejection_reason: Some(
            "schema_version schema-v1 does not match required schema-v2".to_string(),
        ),
    }
}

pub(super) fn ddl_release_gate(
    name: &str,
    satisfied: bool,
    evidence: &str,
) -> DdlBarrierReleaseGate {
    DdlBarrierReleaseGate {
        name: name.to_string(),
        satisfied,
        evidence: evidence.to_string(),
    }
}

pub(super) fn ddl_release_blocker(
    code: &str,
    message: &str,
    sinks: &[&str],
) -> DdlBarrierReleaseBlocker {
    DdlBarrierReleaseBlocker {
        code: code.to_string(),
        message: message.to_string(),
        sinks: sinks.iter().map(|sink| (*sink).to_string()).collect(),
        evidence: "barrier_id=ddl-barrier-abc barrier_lsn=0/16B8000 schema_version=schema-v2"
            .to_string(),
    }
}

pub(super) fn ddl_release_action(
    code: &str,
    sinks: &[&str],
) -> trellara_checkpoint::DdlBarrierReleaseAction {
    trellara_checkpoint::DdlBarrierReleaseAction {
        code: code.to_string(),
        sinks: sinks.iter().map(|sink| (*sink).to_string()).collect(),
        command: "trellara schema ddl-barrier ack --config <flow>".to_string(),
        reason: "record sink evidence before post-DDL DML release".to_string(),
    }
}

pub(super) fn ddl_sink_evidence(
    sink: &str,
    status: &str,
    ack_lsn: Option<&str>,
    release_eligible: bool,
) -> DdlBarrierSinkEvidence {
    DdlBarrierSinkEvidence {
        sink: sink.to_string(),
        status: status.to_string(),
        ack_lsn: ack_lsn.map(ToString::to_string),
        schema_version: ack_lsn.map(|_| "schema-v2".to_string()),
        accepted: ack_lsn.map(|_| true),
        detail: ack_lsn.map(|_| "target schema fingerprint matched".to_string()),
        release_eligible,
        rejection_code: None,
        rejection_reason: None,
    }
}
