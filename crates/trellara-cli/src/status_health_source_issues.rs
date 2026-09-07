use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};

use crate::{
    source_postgres_risk_factors, source_slot_failover_factors, source_wal_retention_factor,
    subscription_conflict_factors, FlowAlertSeverity,
};

pub(crate) fn source_slot_issues(source_slot: &ReplicationSlotStatus) -> Vec<String> {
    let mut issues = source_slot
        .issues
        .iter()
        .map(|issue| format!("source slot {}: {issue}", source_slot.slot_name))
        .collect::<Vec<_>>();
    issues.extend(
        source_slot_failover_factors(source_slot)
            .into_iter()
            .map(|factor| factor.evidence),
    );
    issues.extend(
        source_postgres_risk_factors(source_slot)
            .into_iter()
            .map(|factor| factor.evidence),
    );
    issues
}

pub(crate) fn collect_subscription_issues(
    issues: &mut Vec<String>,
    subscription_conflicts: &[SubscriptionConflictStats],
) -> bool {
    let subscription_factors = subscription_conflict_factors(subscription_conflicts);
    let subscription_conflict_blocked = subscription_factors
        .iter()
        .any(|factor| factor.severity == FlowAlertSeverity::Critical);
    issues.extend(
        subscription_factors
            .into_iter()
            .map(|factor| factor.evidence),
    );
    subscription_conflict_blocked
}

pub(crate) fn collect_wal_retention_issue(
    issues: &mut Vec<String>,
    source_slot: &ReplicationSlotStatus,
    source_wal_retention_warn_bytes: Option<i64>,
) {
    if let Some(factor) = source_wal_retention_factor(
        source_slot,
        source_wal_retention_warn_bytes,
        "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
    ) {
        issues.push(factor.evidence);
    }
}
