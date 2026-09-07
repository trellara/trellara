use crate::{CorrectnessProofCheck, FlowStatusSummary, SnapshotHandoffProofStatus};

const VERIFIED_EVIDENCE_REPAIR: &str =
    "rerun trellara status --config <flow>; if evidence is still inconsistent, rerun trellara snapshot --config <flow> --force";

pub(crate) fn snapshot_handoff_proof_check(
    status: &FlowStatusSummary,
    proof_status: SnapshotHandoffProofStatus,
) -> CorrectnessProofCheck {
    if proof_status.is_verified() {
        let (Some(run), Some(handoff)) = (
            status.latest_snapshot_run.as_ref(),
            status.latest_snapshot_handoff.as_ref(),
        ) else {
            return CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                "snapshot handoff was marked verified but required evidence is missing",
                VERIFIED_EVIDENCE_REPAIR,
            );
        };
        let Some(consistent_lsn) = run.consistent_lsn.as_ref() else {
            return CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                format!(
                    "snapshot run {} was marked verified but has no consistent LSN",
                    run.run_id
                ),
                VERIFIED_EVIDENCE_REPAIR,
            );
        };
        return CorrectnessProofCheck::verified(
            "snapshot_handoff",
            format!(
                "snapshot run {} reached {} at consistent LSN {} and latest handoff for {} uses the same watermark",
                run.run_id,
                run.state,
                consistent_lsn,
                handoff.relation
            ),
        );
    }

    match (
        proof_status,
        status.latest_snapshot_run.as_ref(),
        status.latest_snapshot_handoff.as_ref(),
    ) {
        (SnapshotHandoffProofStatus::MissingRun, None, _) => {
            CorrectnessProofCheck::missing_evidence(
                "snapshot_handoff",
                "snapshot run evidence is missing",
                proof_status.recommendation(),
            )
        }
        (SnapshotHandoffProofStatus::MissingHandoffEvent, Some(run), None) => {
            CorrectnessProofCheck::missing_evidence(
                "snapshot_handoff",
                format!(
                    "snapshot run {} is in {} but no handoff event has been recorded",
                    run.run_id, run.state
                ),
                proof_status.recommendation(),
            )
        }
        (SnapshotHandoffProofStatus::FailedRecoverable, Some(run), handoff) => {
            CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                format!(
                    "snapshot run {} failed recoverably while copying {}; failure_reason={}; latest_handoff={}",
                    run.run_id,
                    run.current_relation.as_deref().unwrap_or("unknown"),
                    run.failure_reason.as_deref().unwrap_or("missing"),
                    handoff
                        .map(|handoff| format!("{} at {}", handoff.relation, handoff.watermark_lsn))
                        .unwrap_or_else(|| "missing".to_string())
                ),
                proof_status.recommendation(),
            )
        }
        (SnapshotHandoffProofStatus::IncompleteRunState, Some(run), Some(handoff)) => {
            CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                format!(
                    "snapshot run {} is still in {} while latest table handoff for {} is recorded at {}",
                    run.run_id, run.state, handoff.relation, handoff.watermark_lsn
                ),
                proof_status.recommendation(),
            )
        }
        (
            SnapshotHandoffProofStatus::HandoffRecordedAwaitingStreamReplay,
            Some(run),
            Some(handoff),
        ) => CorrectnessProofCheck::at_risk(
            "snapshot_handoff",
            format!(
                "snapshot run {} recorded stream_handoff_ready at consistent LSN {:?}; latest handoff for {} is at {}, but stream replay has not yet been observed",
                run.run_id, run.consistent_lsn, handoff.relation, handoff.watermark_lsn
            ),
            proof_status.recommendation(),
        ),
        (SnapshotHandoffProofStatus::MissingConsistentLsn, Some(run), Some(handoff)) => {
            CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                format!(
                    "snapshot run {} reached {} but has no consistent LSN to compare with latest handoff for {} at {}",
                    run.run_id, run.state, handoff.relation, handoff.watermark_lsn
                ),
                proof_status.recommendation(),
            )
        }
        (SnapshotHandoffProofStatus::IdentityMismatch, Some(run), Some(handoff)) => {
            CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                format!(
                    "snapshot handoff evidence identity does not match status flow source_id={} dataset_id={}; run has source_id={} dataset_id={} and latest handoff has source_id={} dataset_id={}",
                    status.source_id,
                    status.dataset_id,
                    run.source_id,
                    run.dataset_id,
                    handoff.source_id,
                    handoff.dataset_id
                ),
                proof_status.recommendation(),
            )
        }
        (SnapshotHandoffProofStatus::WatermarkMismatch, Some(run), Some(handoff)) => {
            CorrectnessProofCheck::at_risk(
                "snapshot_handoff",
                format!(
                    "snapshot run {} reached {} with consistent LSN {:?}, but latest handoff for {} uses watermark {}",
                    run.run_id, run.state, run.consistent_lsn, handoff.relation, handoff.watermark_lsn
                ),
                proof_status.recommendation(),
            )
        }
        _ => CorrectnessProofCheck::at_risk(
            "snapshot_handoff",
            "snapshot handoff evidence is internally inconsistent",
            proof_status.recommendation(),
        ),
    }
}
