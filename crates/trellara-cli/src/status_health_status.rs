use crate::{
    source_postgres_risk_factors, source_slot_wal_boundary_broken, FlowAlertSeverity,
    FlowHealthParts, FlowHealthStatus,
};

pub(crate) fn flow_health_status(
    parts: &FlowHealthParts<'_>,
    issues_empty: bool,
    subscription_conflict_blocked: bool,
) -> FlowHealthStatus {
    if issues_empty {
        FlowHealthStatus::Healthy
    } else if is_blocked(parts, subscription_conflict_blocked) {
        FlowHealthStatus::Blocked
    } else {
        FlowHealthStatus::Degraded
    }
}

fn is_blocked(parts: &FlowHealthParts<'_>, subscription_conflict_blocked: bool) -> bool {
    parts.latest_quarantine.is_some()
        || !parts.source_slot.exists
        || source_slot_wal_boundary_broken(parts.source_slot)
        || subscription_conflict_blocked
        || source_postgres_risk_factors(parts.source_slot)
            .iter()
            .any(|factor| factor.severity == FlowAlertSeverity::Critical)
}
