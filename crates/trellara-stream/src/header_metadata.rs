use trellara_protocol::TransactionEnvelope;

pub(crate) fn ddl_release_gates_header(envelope: &TransactionEnvelope) -> String {
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

pub(crate) fn schema_versions_header(envelope: &TransactionEnvelope) -> String {
    let mut versions = envelope
        .schema_versions
        .iter()
        .filter_map(|version| {
            version
                .relation
                .as_ref()
                .map(|relation| format!("{}={}", relation.display_name(), version.version))
        })
        .collect::<Vec<_>>();
    versions.sort();
    versions.join(",")
}
