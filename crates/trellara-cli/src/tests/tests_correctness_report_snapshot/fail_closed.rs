use super::*;

#[test]
fn snapshot_handoff_proof_fails_closed_when_verified_evidence_is_missing() {
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: None,
        latest_snapshot_run: None,
        ..clean_status_parts()
    });

    let check = snapshot_handoff_proof_check(&status, SnapshotHandoffProofStatus::Verified);

    assert_eq!(check.status, CorrectnessProofStatus::AtRisk);
    assert!(check
        .evidence
        .contains("marked verified but required evidence is missing"));
    assert!(check
        .recommendation
        .as_deref()
        .is_some_and(|recommendation| recommendation.contains("rerun trellara status")));
}

#[test]
fn snapshot_handoff_proof_fails_closed_when_verified_lsn_is_missing() {
    let mut run = latest_snapshot_run();
    run.consistent_lsn = None;
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(latest_snapshot_handoff()),
        latest_snapshot_run: Some(run),
        ..clean_status_parts()
    });

    let check = snapshot_handoff_proof_check(&status, SnapshotHandoffProofStatus::Verified);

    assert_eq!(check.status, CorrectnessProofStatus::AtRisk);
    assert!(check
        .evidence
        .contains("marked verified but has no consistent LSN"));
    assert!(check
        .recommendation
        .as_deref()
        .is_some_and(|recommendation| recommendation.contains("snapshot --config")));
}

#[test]
fn snapshot_handoff_proof_fails_closed_when_verified_identity_mismatches() {
    let mut handoff = latest_snapshot_handoff();
    handoff.source_id = "source-b".to_string();
    let status = FlowStatusSummary::new(FlowStatusParts {
        latest_snapshot_handoff: Some(handoff),
        latest_snapshot_run: Some(latest_snapshot_run()),
        ..clean_status_parts()
    });

    let check = snapshot_handoff_proof_check(&status, SnapshotHandoffProofStatus::IdentityMismatch);

    assert_eq!(check.status, CorrectnessProofStatus::AtRisk);
    assert!(check.evidence.contains("identity does not match"));
    assert!(check.evidence.contains("source_id=source-a"));
    assert!(check.evidence.contains("source_id=source-b"));
}
