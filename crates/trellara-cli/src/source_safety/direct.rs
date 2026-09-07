use trellara_pg_capture::{ReplicationSlotStatus, SubscriptionConflictStats};

use crate::{
    source_postgres_risk_factors, source_safety::types::*, source_slot_failover_factors,
    source_slot_inactive_evidence, source_slot_inactive_recommendation, source_slot_issue_factors,
    source_slot_position_evidence, source_table_factors, source_table_is_unsafe,
    source_wal_retention_factor, subscription_conflict_factors, SourceSafetyInitRecommendation,
    SourceSafetyScore,
};

impl DirectSourceSafetySummary {
    pub(crate) fn from_source_inspection(
        source_id: String,
        dataset_id: String,
        wal_retention_warn_bytes: Option<i64>,
        tables: Vec<trellara_pg_capture::TablePreflight>,
        slot: ReplicationSlotStatus,
        subscription_conflicts: Vec<SubscriptionConflictStats>,
        init_recommendation: Option<SourceSafetyInitRecommendation>,
    ) -> Self {
        let mut factors = Vec::new();

        for table in &tables {
            factors.extend(source_table_factors(table));
        }

        if slot.exists {
            if !slot.issues.is_empty() {
                factors.extend(source_slot_issue_factors(&slot));
            } else {
                factors.extend(source_slot_failover_factors(&slot));
            }
            if slot.issues.is_empty() && slot.active == Some(false) {
                factors.push(SourceSafetyFactor::warning(
                    "source_slot_inactive",
                    10,
                    source_slot_inactive_evidence(&slot),
                    source_slot_inactive_recommendation(&slot),
                ));
            }
        } else {
            factors.push(SourceSafetyFactor::warning(
                "source_slot_not_created",
                5,
                format!(
                    "source replication slot {} does not exist yet; {}",
                    slot.slot_name,
                    source_slot_position_evidence(&slot)
                ),
                "run trellara bootstrap only after source-safety and preflight are acceptable",
            ));
        }

        factors.extend(source_postgres_risk_factors(&slot));
        factors.extend(subscription_conflict_factors(&subscription_conflicts));

        if let Some(factor) = source_wal_retention_factor(
            &slot,
            wal_retention_warn_bytes,
            "drain CDC lag or drop abandoned slots before source WAL retention grows further",
        ) {
            factors.push(factor);
        }

        let score = SourceSafetyScore::from_factors(&factors);
        let unsafe_table_count = tables
            .iter()
            .filter(|table| source_table_is_unsafe(table))
            .count();

        Self {
            source_id,
            dataset_id,
            mode: "source_only_readiness".to_string(),
            read_only: true,
            score: score.score,
            grade: score.grade,
            status: score.status,
            table_count: tables.len(),
            unsafe_table_count,
            slot,
            subscription_conflicts,
            factor_count: score.factor_count,
            critical_factor_count: score.critical_factor_count,
            warning_factor_count: score.warning_factor_count,
            factors,
            recommended_actions: score.recommended_actions,
            init_recommendation,
            init_config_written: None,
        }
    }
}
