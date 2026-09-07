use crate::fleet_fanin::{FleetFanInQuarantineEntry, FleetFanInSimulationReport};
use trellara_lake::{LakeCompletenessState, LakeEpochVerificationStatus, LakeError};

pub(crate) fn mark_conflicting_duplicate_quarantine(
    report: &mut FleetFanInSimulationReport,
    error: &LakeError,
    quarantined_envelope: Option<(String, String, String)>,
) {
    report.initial_state = LakeCompletenessState::Quarantined;
    report.complete_source_count = report.complete_source_count.saturating_sub(1);
    report.quarantined_source_count = 1;
    report.verification_status = LakeEpochVerificationStatus::Unknown;
    if let Some(source) = report.source_watermarks.first_mut() {
        source.state = "quarantined".to_string();
        source.gap_reason = Some("conflicting duplicate idempotency evidence".to_string());
    }
    if let Some((source_id, transaction_id, commit_lsn)) = quarantined_envelope {
        report.quarantine_entries.push(FleetFanInQuarantineEntry {
            source_id,
            transaction_id: Some(transaction_id),
            commit_lsn: Some(commit_lsn),
            reason: "conflicting_duplicate_idempotency".to_string(),
            details: error.to_string(),
            recovery_command: "trellara lake fanin verify --config <config> --stream-epoch <stream-epoch.json> --lake-epoch <lake-epoch.json>".to_string(),
        });
    }
}
