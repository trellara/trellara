use serde::Serialize;
use trellara_pg_capture::{
    DatabaseSourceSafetyInspection, ReplicationSlotStatus, SubscriptionConflictStats,
};

use crate::{factors::factors_for_inspection, scoring::score_from_factors};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckSummary {
    pub tool: String,
    pub database: String,
    pub read_only: bool,
    pub managed_postgres_ready: String,
    pub score: u8,
    pub grade: CheckGrade,
    pub status: CheckStatus,
    pub table_count: usize,
    pub unsafe_table_count: usize,
    pub logical_slot_count: usize,
    pub at_risk_slot_count: usize,
    pub subscription_conflict_count: usize,
    pub inspection_warnings: Vec<String>,
    pub unsafe_tables: Vec<String>,
    pub logical_slots: Vec<ReplicationSlotStatus>,
    pub subscription_conflicts: Vec<SubscriptionConflictStats>,
    pub findings: Vec<CheckFactor>,
    pub recommended_actions: Vec<String>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckStatus {
    Healthy,
    Degraded,
    Blocked,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckGrade {
    A,
    B,
    C,
    D,
    F,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSeverity {
    Warning,
    Critical,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CheckFactor {
    pub code: String,
    pub severity: CheckSeverity,
    pub points_lost: u8,
    pub evidence: String,
    pub recommendation: String,
}

impl CheckSummary {
    pub fn from_inspection(database: String, inspection: DatabaseSourceSafetyInspection) -> Self {
        let findings = factors_for_inspection(
            &inspection.tables,
            &inspection.logical_slots,
            &inspection.subscription_conflicts,
            &inspection.inspection_warnings,
        );
        let score = score_from_factors(&findings);
        let unsafe_tables = inspection
            .tables
            .iter()
            .filter(|table| !table.exists || !table.update_delete_safe)
            .map(|table| table.qualified_name())
            .collect::<Vec<_>>();
        let at_risk_slot_count = inspection
            .logical_slots
            .iter()
            .filter(|slot| {
                findings
                    .iter()
                    .any(|factor| factor_mentions_slot(factor, &slot.slot_name))
            })
            .count();
        let recommended_actions = dedup_actions(&findings);

        Self {
            tool: "trellara-check".to_string(),
            database,
            read_only: inspection.read_only,
            managed_postgres_ready:
                "normal PostgreSQL TLS connection; no config, publication, slot creation, replication socket, or writes"
                    .to_string(),
            score: score.score,
            grade: score.grade,
            status: score.status,
            table_count: inspection.tables.len(),
            unsafe_table_count: unsafe_tables.len(),
            logical_slot_count: inspection.logical_slots.len(),
            at_risk_slot_count,
            subscription_conflict_count: inspection.subscription_conflicts.len(),
            inspection_warnings: inspection.inspection_warnings,
            unsafe_tables,
            logical_slots: inspection.logical_slots,
            subscription_conflicts: inspection.subscription_conflicts,
            findings,
            recommended_actions,
        }
    }
}

impl CheckFactor {
    pub fn warning(
        code: impl Into<String>,
        points_lost: u8,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: CheckSeverity::Warning,
            points_lost,
            evidence: evidence.into(),
            recommendation: recommendation.into(),
        }
    }

    pub fn critical(
        code: impl Into<String>,
        points_lost: u8,
        evidence: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            severity: CheckSeverity::Critical,
            points_lost,
            evidence: evidence.into(),
            recommendation: recommendation.into(),
        }
    }
}

fn dedup_actions(findings: &[CheckFactor]) -> Vec<String> {
    let mut actions = Vec::new();
    for finding in findings {
        if !actions.contains(&finding.recommendation) {
            actions.push(finding.recommendation.clone());
        }
    }
    actions
}

fn factor_mentions_slot(factor: &CheckFactor, slot_name: &str) -> bool {
    [
        format!("source slot {slot_name}"),
        format!("source replication slot {slot_name}"),
        format!("slot: {slot_name}"),
    ]
    .iter()
    .any(|needle| factor.evidence.contains(needle))
}
