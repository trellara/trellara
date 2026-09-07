use trellara_protocol::{
    ddl_propagation_policy_sha256, summarize_ddl_propagation, TransactionEnvelope,
};
use trellara_stream::StreamHeader;

use crate::barrier_header_lookup::required_header;
use crate::barrier_header_payload::validate_header_payload_field;
use crate::ApplyWorkerResult;

pub(crate) fn validate_strict_ddl_proof_headers(
    headers: &[StreamHeader],
    envelope: &TransactionEnvelope,
) -> ApplyWorkerResult<()> {
    if envelope.ddl_events.is_empty() {
        return Ok(());
    }

    let summary = summarize_ddl_propagation(&envelope.ddl_events)?;
    validate_required_ddl_header(
        headers,
        "ddl_release_gates",
        "trellara.ddl_release_gates",
        &ddl_release_gates(envelope),
    )?;
    validate_required_ddl_header(
        headers,
        "ddl_propagation_decisions",
        "trellara.ddl_propagation_decisions",
        &summary.evidence(),
    )?;
    validate_required_ddl_header(
        headers,
        "ddl_target_ack_required",
        "trellara.ddl_target_ack_required",
        &summary.target_ack_required.to_string(),
    )?;
    validate_required_ddl_header(
        headers,
        "ddl_propagation_policy_sha256",
        "trellara.ddl_propagation_policy_sha256",
        &ddl_propagation_policy_sha256(&envelope.ddl_events)?,
    )
}

fn validate_required_ddl_header(
    headers: &[StreamHeader],
    field: &'static str,
    key: &'static str,
    expected: &str,
) -> ApplyWorkerResult<()> {
    let header = required_header(headers, key)?;
    validate_header_payload_field(field, &header, expected)
}

fn ddl_release_gates(envelope: &TransactionEnvelope) -> String {
    let mut gates = envelope
        .ddl_events
        .iter()
        .map(|event| event.release_gate.clone())
        .filter(|gate| !gate.trim().is_empty())
        .collect::<Vec<_>>();
    gates.sort();
    gates.dedup();
    gates.join(",")
}
