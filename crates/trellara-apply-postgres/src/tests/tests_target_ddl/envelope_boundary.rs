use super::envelope_apply::{
    blocked_barrier_summary, released_barrier_summary, target_ack_evidence,
};
use super::*;

#[test]
fn ddl_dml_replay_proof_rejects_blocked_or_mismatched_boundaries() {
    let blocked = target_ddl_release_decision(&blocked_barrier_summary());
    let dml = ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6C50".to_string(),
    };
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &target_ack_evidence(),
            &blocked,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers == vec![
                    "pending required sink acknowledgements: raw_cdc_lake".to_string()
                ]
    ));

    let released = target_ddl_release_decision(&released_barrier_summary());
    let wrong_dml = ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 1,
        commit_lsn: "0/16B6D00".to_string(),
    };
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &target_ack_evidence(),
            &released,
            &wrong_dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("does not match DDL barrier_lsn 0/16B6C50")
    ));

    let mut stale_ack = target_ack_evidence();
    stale_ack.ack_lsn = "0/16B6B00".to_string();
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(&stale_ack, &released, &dml),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("target ACK LSN 0/16B6B00 does not match barrier LSN 0/16B6C50")
    ));

    let mut missing_ack_barrier = target_ack_evidence();
    missing_ack_barrier.barrier_lsn = None;
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &missing_ack_barrier,
            &released,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("missing canonical barrier_lsn evidence")
    ));

    let mut wrong_ack_barrier_lsn = target_ack_evidence();
    wrong_ack_barrier_lsn.barrier_lsn = Some("0/16B6D00".to_string());
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &wrong_ack_barrier_lsn,
            &released,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("target ACK barrier LSN 0/16B6D00 does not match barrier LSN 0/16B6C50")
    ));

    let mut wrong_barrier_ack = target_ack_evidence();
    wrong_barrier_ack.barrier_id = "ddl-barrier-other".to_string();
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &wrong_barrier_ack,
            &released,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("target ACK barrier_id ddl-barrier-other")
                && blockers[0].contains("release decision barrier_id ddl-barrier-123")
    ));

    let mut wrong_database_ack = target_ack_evidence();
    wrong_database_ack.database_id = "analytics".to_string();
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &wrong_database_ack,
            &released,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("target ACK database_id analytics")
                && blockers[0].contains("release decision database_id retail")
    ));

    let mut wrong_sink_ack = target_ack_evidence();
    wrong_sink_ack.sink = "raw_cdc_lake".to_string();
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &wrong_sink_ack,
            &released,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("target ACK sink raw_cdc_lake")
    ));

    let mut wrong_gate_ack = target_ack_evidence();
    wrong_gate_ack.release_gate = "manual_release".to_string();
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &wrong_gate_ack,
            &released,
            &dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("release gate manual_release")
    ));

    let duplicate_only_dml = ApplyOutcome {
        decision: ApplyDecision::SkippedDuplicate,
        applied_changes: 0,
        commit_lsn: "0/16B6C50".to_string(),
    };
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &target_ack_evidence(),
            &released,
            &duplicate_only_dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("must apply at least one change")
                && blockers[0].contains("SkippedDuplicate")
    ));

    let zero_change_dml = ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: 0,
        commit_lsn: "0/16B6C50".to_string(),
    };
    assert!(matches!(
        crate::ddl_dml_replay_proof::target_ddl_dml_replay_proof(
            &target_ack_evidence(),
            &released,
            &zero_change_dml,
        ),
        Err(ApplyError::DdlDmlReleaseBlocked { barrier_id, blockers })
            if barrier_id == "ddl-barrier-123"
                && blockers[0].contains("must apply at least one change")
                && blockers[0].contains("applied_changes=0")
    ));
}
