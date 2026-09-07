use crate::{
    LocalStreamLocateBoundarySummary, QuarantineReplayBoundaryEvidence,
    QuarantineReplaySafetyContract,
};

pub(crate) fn quarantine_replay_safety_contract_with_boundary(
    dedup_removed: bool,
    quarantine_cleared: bool,
    boundary: Option<&LocalStreamLocateBoundarySummary>,
    exact_seek_command_count: usize,
) -> QuarantineReplaySafetyContract {
    let evidence = boundary_evidence(boundary, exact_seek_command_count);
    let exact_seek_available = evidence.exact_seek_available;

    QuarantineReplaySafetyContract {
        exact_boundary_required: true,
        cleared_state: cleared_state(dedup_removed, quarantine_cleared),
        redelivery_gate: redelivery_gate(exact_seek_available),
        next_operator_action: next_operator_action(exact_seek_available),
        boundary_evidence: evidence,
    }
}

fn boundary_evidence(
    boundary: Option<&LocalStreamLocateBoundarySummary>,
    exact_seek_command_count: usize,
) -> QuarantineReplayBoundaryEvidence {
    let exact_seek_available = exact_seek_command_count > 0;
    match boundary {
        Some(boundary) => QuarantineReplayBoundaryEvidence {
            exact_seek_available,
            boundary_status: boundary.status.clone(),
            boundary_mode: Some(boundary.mode.clone()),
            complete: Some(boundary.complete),
            exact_seek_command_count,
            required_message_kinds: boundary.required_message_kinds.clone(),
            missing_message_kinds: boundary.missing_message_kinds.clone(),
            found_partition_ids: boundary.found_partition_ids.clone(),
            missing_partition_ids: boundary.missing_partition_ids.clone(),
            metadata_conflicts: boundary.metadata_conflicts.clone(),
        },
        None => QuarantineReplayBoundaryEvidence {
            exact_seek_available,
            boundary_status: fallback_boundary_status(exact_seek_available),
            boundary_mode: None,
            complete: None,
            exact_seek_command_count,
            required_message_kinds: Vec::new(),
            missing_message_kinds: Vec::new(),
            found_partition_ids: Vec::new(),
            missing_partition_ids: Vec::new(),
            metadata_conflicts: Vec::new(),
        },
    }
}

fn fallback_boundary_status(exact_seek_available: bool) -> String {
    if exact_seek_available {
        "located"
    } else {
        "not_located"
    }
    .to_string()
}

fn cleared_state(dedup_removed: bool, quarantine_cleared: bool) -> String {
    match (dedup_removed, quarantine_cleared) {
        (true, true) => "dedup and quarantine rows cleared for the exact transaction boundary",
        (true, false) => {
            "dedup row cleared; quarantine row was already absent at the exact boundary"
        }
        (false, true) => "quarantine row cleared; no applied transaction dedup row existed",
        (false, false) => "no local target state was cleared after exact-boundary lookup",
    }
    .to_string()
}

fn redelivery_gate(exact_seek_available: bool) -> String {
    if exact_seek_available {
        "seek the durable stream to the located transaction boundary before replay"
    } else {
        "locate the durable stream transaction boundary before replay"
    }
    .to_string()
}

fn next_operator_action(exact_seek_available: bool) -> String {
    if exact_seek_available {
        "run the emitted exact seek command, then restart apply for this flow"
    } else {
        "run the emitted locate command, seek to the returned offset, then restart apply"
    }
    .to_string()
}
