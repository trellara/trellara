use crate::DdlBarrierSinkEvidence;

pub(crate) struct DdlBarrierBlockerContext<'a> {
    pub(crate) barrier_id: &'a str,
    pub(crate) barrier_lsn: &'a str,
    pub(crate) schema_version: &'a str,
    pub(crate) has_required_sinks: bool,
    pub(crate) requires_global_partition_pause: bool,
}

pub(crate) fn ddl_release_blockers(
    has_required_sinks: bool,
    pending_sinks: &[String],
    rejected_sinks: &[String],
    unexpected_sinks: &[String],
) -> Vec<String> {
    let mut blockers = Vec::new();
    if !has_required_sinks {
        blockers.push("missing required sink acknowledgements contract".to_string());
    }
    if !pending_sinks.is_empty() {
        blockers.push(format!(
            "pending required sink acknowledgements: {}",
            pending_sinks.join(", ")
        ));
    }
    if !rejected_sinks.is_empty() {
        blockers.push(format!(
            "rejected or stale sink acknowledgements: {}",
            rejected_sinks.join(", ")
        ));
    }
    if !unexpected_sinks.is_empty() {
        blockers.push(format!(
            "unexpected sink acknowledgements: {}",
            unexpected_sinks.join(", ")
        ));
    }
    blockers
}

pub(crate) fn ddl_release_blocker_codes(
    has_required_sinks: bool,
    pending_sinks: &[String],
    sink_evidence: &[DdlBarrierSinkEvidence],
    release_dml: bool,
    requires_global_partition_pause: bool,
) -> Vec<String> {
    let mut codes = Vec::new();
    if !has_required_sinks {
        codes.push("missing_required_sinks".to_string());
    }
    if !pending_sinks.is_empty() {
        codes.push("pending_required_ack".to_string());
    }
    if requires_global_partition_pause && !release_dml {
        codes.push("partition_visibility_not_released".to_string());
    }
    codes.extend(
        sink_evidence
            .iter()
            .filter_map(|evidence| evidence.rejection_code.clone()),
    );
    codes.sort();
    codes.dedup();
    codes
}
