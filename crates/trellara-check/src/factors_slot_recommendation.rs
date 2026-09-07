use trellara_pg_capture::ReplicationSlotStatus;

pub(crate) fn source_slot_issue_recommendation(slot: &ReplicationSlotStatus) -> &'static str {
    if slot
        .issues
        .iter()
        .any(|issue| issue.contains("WAL is lost") || issue.contains("invalidated"))
    {
        "recreate the source replication slot, reseed affected targets, then resume CDC from the fresh handoff"
    } else if slot
        .issues
        .iter()
        .any(|issue| issue.contains("unreserved") || issue.contains("safe WAL size is exhausted"))
    {
        "pause unsafe consumers, drain relay/apply lag, or reseed affected targets before the source slot loses WAL"
    } else {
        "inspect source preflight output and repair the replication configuration"
    }
}
