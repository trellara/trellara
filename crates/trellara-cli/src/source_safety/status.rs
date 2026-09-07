pub(crate) mod factors;

use crate::{
    source_postgres_risk_factors,
    source_safety::status::factors::{
        status_partition_factors, status_recovery_factors, status_target_factors,
    },
    source_slot_failover_factors, source_slot_issue_factors, source_slot_position_evidence,
    source_wal_retention_factor, subscription_conflict_factors, FlowStatusSummary,
    SourceSafetyFactor, SourceSafetyScore, SourceSafetySummary,
};

impl SourceSafetySummary {
    pub(crate) fn from_status(status: FlowStatusSummary) -> Self {
        let mut factors = Vec::new();

        if !status.source_slot.exists {
            factors.push(SourceSafetyFactor::critical(
                "source_slot_missing",
                35,
                format!(
                    "source replication slot {} is missing; {}",
                    status.source_slot.slot_name,
                    source_slot_position_evidence(&status.source_slot)
                ),
                "run trellara bootstrap to create or repair the source replication slot",
            ));
        } else if !status.source_slot.issues.is_empty() {
            factors.extend(source_slot_issue_factors(&status.source_slot));
        } else {
            factors.extend(source_slot_failover_factors(&status.source_slot));
        }

        factors.extend(source_postgres_risk_factors(&status.source_slot));
        if let Some(factor) = source_wal_retention_factor(
            &status.source_slot,
            status.source_wal_retention_warn_bytes,
            "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
        ) {
            factors.push(factor);
        }

        factors.extend(subscription_conflict_factors(
            &status.subscription_conflicts,
        ));

        match status.source.as_ref() {
            Some(source) if !source.source_is_durable => {
                factors.push(SourceSafetyFactor::warning(
                    "source_checkpoint_lag",
                    20,
                    format!(
                        "source durable LSN {} is behind seen LSN {} by {} bytes",
                        source.last_durable_lsn, source.last_seen_lsn, source.seen_to_durable_bytes
                    ),
                    "run trellara relay until source durable checkpoint catches up",
                ));
            }
            None => factors.push(SourceSafetyFactor::critical(
                "source_checkpoint_missing",
                25,
                "source checkpoint is missing",
                "run trellara relay after bootstrap to establish a source checkpoint",
            )),
            _ => {}
        }

        factors.extend(status_target_factors(&status));
        factors.extend(status_partition_factors(&status));
        factors.extend(status_recovery_factors(&status));

        let score = SourceSafetyScore::from_factors(&factors);

        Self {
            source_id: status.source_id,
            dataset_id: status.dataset_id,
            mode: status.mode,
            score: score.score,
            grade: score.grade,
            status: score.status,
            slot: status.source_slot,
            subscription_conflicts: status.subscription_conflicts,
            factor_count: score.factor_count,
            critical_factor_count: score.critical_factor_count,
            warning_factor_count: score.warning_factor_count,
            factors,
            recommended_actions: score.recommended_actions,
        }
    }
}
