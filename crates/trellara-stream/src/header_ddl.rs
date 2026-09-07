use trellara_protocol::{
    ddl_propagation_policy_sha256, summarize_ddl_propagation, TransactionEnvelope,
};

use crate::header_metadata::ddl_release_gates_header;
use crate::StreamHeader;

pub(crate) fn ddl_headers(envelope: &TransactionEnvelope) -> Vec<StreamHeader> {
    if envelope.ddl_events.is_empty() {
        return Vec::new();
    }

    let mut headers = vec![StreamHeader::new(
        "trellara.ddl_release_gates",
        ddl_release_gates_header(envelope),
    )];
    if let Ok(summary) = summarize_ddl_propagation(&envelope.ddl_events) {
        headers.push(StreamHeader::new(
            "trellara.ddl_propagation_decisions",
            summary.evidence(),
        ));
        headers.push(StreamHeader::new(
            "trellara.ddl_target_ack_required",
            summary.target_ack_required.to_string(),
        ));
    }
    if let Ok(policy_sha256) = ddl_propagation_policy_sha256(&envelope.ddl_events) {
        headers.push(StreamHeader::new(
            "trellara.ddl_propagation_policy_sha256",
            policy_sha256,
        ));
    }
    headers
}
