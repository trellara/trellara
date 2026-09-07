use crate::{
    source_postgres_risk_factors, source_slot_failover_factors, source_slot_issue_factors,
    source_wal_retention_factor, strongest_source_factor, FlowAlertSeverity, FlowFailureParts,
    FlowFailureSummary,
};

pub(crate) fn source_failure_from_parts(
    parts: &FlowFailureParts<'_>,
    recovery_action_codes: &[String],
) -> Option<FlowFailureSummary> {
    if !parts.source_slot.exists {
        return Some(FlowFailureSummary::critical(
            "source_slot_missing",
            format!(
                "source replication slot {} is missing",
                parts.source_slot.slot_name
            ),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    let source_slot_factors = if parts.source_slot.issues.is_empty() {
        Vec::new()
    } else {
        source_slot_issue_factors(parts.source_slot)
    };
    if let Some(factor) = source_slot_factors
        .iter()
        .find(|factor| factor.severity == FlowAlertSeverity::Critical)
    {
        return Some(FlowFailureSummary::critical(
            factor.code.clone(),
            factor.evidence.clone(),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    if let Some(drift) = parts.source_schema_drift {
        return Some(FlowFailureSummary::critical(
            "source_schema_handoff_required",
            format!("{} for {}", drift.reason, drift.relations.join(", ")),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    if let Some(factor) = source_slot_factors.first() {
        return Some(FlowFailureSummary::warning(
            factor.code.clone(),
            factor.evidence.clone(),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    if let Some(factor) = source_slot_failover_factors(parts.source_slot).first() {
        return Some(FlowFailureSummary::warning(
            factor.code.clone(),
            factor.evidence.clone(),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    if let Some(factor) = strongest_source_factor(source_postgres_risk_factors(parts.source_slot)) {
        return Some(match factor.severity {
            FlowAlertSeverity::Critical => FlowFailureSummary::critical(
                factor.code,
                factor.evidence,
                None,
                recovery_action_codes.to_vec(),
            ),
            FlowAlertSeverity::Warning => FlowFailureSummary::warning(
                factor.code,
                factor.evidence,
                None,
                recovery_action_codes.to_vec(),
            ),
        });
    }
    if let Some(source) = parts.source.filter(|source| !source.source_is_durable) {
        return Some(FlowFailureSummary::warning(
            "source_checkpoint_lag",
            format!(
                "source durable LSN {} is behind seen LSN {} by {} bytes",
                source.last_durable_lsn, source.last_seen_lsn, source.seen_to_durable_bytes
            ),
            None,
            recovery_action_codes.to_vec(),
        ));
    }
    if let Some(factor) = source_wal_retention_factor(
        parts.source_slot,
        parts.source_wal_retention_warn_bytes,
        "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
    ) {
        return Some(FlowFailureSummary::warning(
            factor.code,
            factor.evidence,
            None,
            recovery_action_codes.to_vec(),
        ));
    }

    None
}
