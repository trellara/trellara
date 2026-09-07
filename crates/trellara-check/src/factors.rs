use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats, TablePreflight};

use crate::{
    factors_postgres::postgres_risk_factors, factors_slots::slot_factors,
    factors_subscription::subscription_conflict_factors, factors_tables::table_factors,
    CheckFactor,
};

pub(crate) fn factors_for_inspection(
    tables: &[TablePreflight],
    slots: &[ReplicationSlotStatus],
    subscription_conflicts: &[SubscriptionConflictStats],
    inspection_warnings: &[String],
) -> Vec<CheckFactor> {
    let mut findings = Vec::new();
    findings.extend(table_factors(tables));
    for slot in slots {
        findings.extend(slot_factors(slot));
    }
    findings.extend(postgres_risk_factors(slots));
    findings.extend(subscription_conflict_factors(subscription_conflicts));
    findings.extend(inspection_warning_factors(inspection_warnings));
    findings
}

fn inspection_warning_factors(warnings: &[String]) -> Vec<CheckFactor> {
    if warnings.is_empty() {
        return Vec::new();
    }
    vec![CheckFactor::warning(
        "inspection_incomplete",
        10,
        warnings.join("; "),
        "rerun with a role that can read replication slot and subscription catalog views, such as a monitoring role on managed PostgreSQL",
    )]
}
