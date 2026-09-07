use crate::{
    DdlPlanChange, DdlPlanDecision, DdlPlanVerdict, DdlPropagationPolicyMode,
    DdlPropagationPolicySummary,
};

pub(crate) fn ddl_propagation_policy_modes(
    verdict: DdlPlanVerdict,
    changes: &[DdlPlanChange],
) -> Vec<DdlPropagationPolicySummary> {
    let auto_apply_count = decision_count(changes, DdlPlanDecision::AutoApply);
    let staged_rollout_count = decision_count(changes, DdlPlanDecision::StageThenApply);
    let manual_review_count = decision_count(changes, DdlPlanDecision::ManualReview);
    let blocked_count = decision_count(changes, DdlPlanDecision::Block);

    vec![
        policy(
            DdlPropagationPolicyMode::AutoApply,
            auto_apply_count,
            "compatible DDL may execute automatically, but post-DDL DML still waits for required sink ACK evidence",
            "target DDL apply-plan digest, statement digests, and required sink ACKs at barrier_lsn",
        ),
        policy(
            DdlPropagationPolicyMode::StagedRollout,
            staged_rollout_count,
            "compatible DDL is staged first; DML release waits for staged rollout validation and sink ACK evidence",
            "staged rollout approval with accepted schema version, validation result, and sink ACKs at barrier_lsn",
        ),
        policy(
            DdlPropagationPolicyMode::ManualApprovalRequired,
            manual_review_count,
            "operator approval must name the accepted schema version, mapping, and target action before DML release",
            "operator approval record with accepted schema version, explicit mapping, target action, and approver identity",
        ),
        policy(
            DdlPropagationPolicyMode::BlockUnsupported,
            blocked_count,
            "unsupported or destructive DDL blocks post-DDL DML until policy, mapping, or schema design is changed",
            "blocker remediation evidence proving the DDL was redesigned, mapped, or explicitly rejected before release",
        ),
        DdlPropagationPolicySummary {
            mode: DdlPropagationPolicyMode::ShadowPlanOnly,
            active: verdict == DdlPlanVerdict::Blocked,
            change_count: blocked_count,
            release_rule: "blocked plans produce review and repair evidence only; no release barrier is recorded until blockers are resolved"
                .to_string(),
            approval_evidence:
                "shadow plan review artifact with blocker list and no post-DDL DML release".to_string(),
        },
    ]
}

fn decision_count(changes: &[DdlPlanChange], decision: DdlPlanDecision) -> usize {
    changes
        .iter()
        .filter(|change| change.decision == decision)
        .count()
}

fn policy(
    mode: DdlPropagationPolicyMode,
    change_count: usize,
    release_rule: &str,
    approval_evidence: &str,
) -> DdlPropagationPolicySummary {
    DdlPropagationPolicySummary {
        mode,
        active: change_count > 0,
        change_count,
        release_rule: release_rule.to_string(),
        approval_evidence: approval_evidence.to_string(),
    }
}
