use crate::{
    status_health_issue_collectors::{
        collect_checkpoint_issues, collect_failure_issues, collect_partition_issues,
    },
    status_health_source_issues::{
        collect_subscription_issues, collect_wal_retention_issue, source_slot_issues,
    },
    FlowHealthParts,
};

pub(crate) struct FlowHealthEvaluation {
    pub(crate) issues: Vec<String>,
    pub(crate) subscription_conflict_blocked: bool,
}

pub(crate) fn evaluate_flow_health(parts: &FlowHealthParts<'_>) -> FlowHealthEvaluation {
    let mut issues = source_slot_issues(parts.source_slot);
    let subscription_conflict_blocked =
        collect_subscription_issues(&mut issues, parts.subscription_conflicts);
    collect_wal_retention_issue(
        &mut issues,
        parts.source_slot,
        parts.source_wal_retention_warn_bytes,
    );
    collect_checkpoint_issues(&mut issues, parts.source, parts.target);
    collect_partition_issues(&mut issues, parts.partition_watermarks);
    collect_failure_issues(
        &mut issues,
        parts.latest_quarantine,
        parts.latest_validation,
        parts.source,
        parts.target,
    );

    FlowHealthEvaluation {
        issues,
        subscription_conflict_blocked,
    }
}
