use crate::{
    source_postgres_risk_factors, source_slot_issue_factors, source_slot_issue_recommendation,
    source_wal_retention_factor, strongest_source_factor, subscription_conflict_factors,
    CorrectnessProofCheck, CorrectnessProofInputs,
};

pub(crate) fn source_proof_checks(
    input: &CorrectnessProofInputs<'_>,
) -> Vec<CorrectnessProofCheck> {
    vec![
        source_slot_proof_check(input),
        source_subscription_conflicts_proof_check(input),
        source_schema_contract_proof_check(input),
        source_wal_retention_proof_check(input),
        source_checkpoint_proof_check(input),
    ]
}

fn source_slot_proof_check(input: &CorrectnessProofInputs<'_>) -> CorrectnessProofCheck {
    let source_slot = &input.status.source_slot;
    if input.source_slot_safe {
        CorrectnessProofCheck::verified(
            "source_slot",
            format!(
                "replication slot {} exists and has no safety issues",
                source_slot.slot_name
            ),
        )
    } else {
        let slot_factor = if source_slot.exists {
            source_slot_issue_factors(source_slot)
                .into_iter()
                .next()
                .or_else(|| strongest_source_factor(source_postgres_risk_factors(source_slot)))
        } else {
            None
        };
        CorrectnessProofCheck::at_risk(
            "source_slot",
            slot_factor
                .as_ref()
                .map(|factor| factor.evidence.clone())
                .or_else(|| source_slot.issues.first().cloned())
                .unwrap_or_else(|| {
                    format!(
                        "replication slot {} is missing or unsafe",
                        source_slot.slot_name
                    )
                }),
            slot_factor
                .as_ref()
                .map(|factor| factor.recommendation.as_str())
                .unwrap_or_else(|| source_slot_issue_recommendation(source_slot)),
        )
    }
}

fn source_subscription_conflicts_proof_check(
    input: &CorrectnessProofInputs<'_>,
) -> CorrectnessProofCheck {
    if input.source_subscription_conflicts_safe {
        CorrectnessProofCheck::verified(
            "source_subscription_conflicts",
            "logical replication subscription conflict counters have no critical conflicts",
        )
    } else {
        let factor = subscription_conflict_factors(&input.status.subscription_conflicts)
            .into_iter()
            .next()
            .expect("checked subscription conflict factors");
        CorrectnessProofCheck::at_risk(
            "source_subscription_conflicts",
            factor.evidence,
            factor.recommendation,
        )
    }
}

fn source_schema_contract_proof_check(input: &CorrectnessProofInputs<'_>) -> CorrectnessProofCheck {
    if input.source_schema_contract_safe {
        CorrectnessProofCheck::verified(
            "source_schema_contract",
            "configured source schema fingerprints match live pgoutput metadata",
        )
    } else {
        let drift = input
            .status
            .source_schema_drift
            .as_ref()
            .expect("checked source schema drift");
        CorrectnessProofCheck::at_risk(
            "source_schema_contract",
            format!("{} for {}", drift.reason, drift.relations.join(", ")),
            drift.recommendation.clone(),
        )
    }
}

fn source_wal_retention_proof_check(input: &CorrectnessProofInputs<'_>) -> CorrectnessProofCheck {
    let status = input.status;
    if input.source_wal_retention_safe {
        CorrectnessProofCheck::verified(
            "source_wal_retention",
            "source slot WAL retention has no projected headroom risk",
        )
    } else {
        let factor = source_wal_retention_factor(
            &status.source_slot,
            status.source_wal_retention_warn_bytes,
            "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
        );
        CorrectnessProofCheck::at_risk(
            "source_wal_retention",
            factor
                .as_ref()
                .map(|factor| factor.evidence.clone())
                .unwrap_or_else(|| {
                    format!(
                        "source slot {} WAL retention is at risk",
                        status.source_slot.slot_name
                    )
                }),
            factor
                .map(|factor| factor.recommendation)
                .unwrap_or_else(|| {
                    "drain relay/apply lag or reseed slow targets before source WAL retention grows further"
                        .to_string()
                }),
        )
    }
}

fn source_checkpoint_proof_check(input: &CorrectnessProofInputs<'_>) -> CorrectnessProofCheck {
    match input.status.source.as_ref() {
        Some(source) if input.source_checkpoint_durable => CorrectnessProofCheck::verified(
            "source_checkpoint",
            format!(
                "source durable checkpoint {} has caught up to seen LSN {}",
                source.last_durable_lsn, source.last_seen_lsn
            ),
        ),
        Some(source) => CorrectnessProofCheck::at_risk(
            "source_checkpoint",
            format!(
                "source durable checkpoint {} is behind seen LSN {} by {} bytes",
                source.last_durable_lsn, source.last_seen_lsn, source.seen_to_durable_bytes
            ),
            "run trellara relay until source durable checkpoint catches up",
        ),
        None => CorrectnessProofCheck::missing_evidence(
            "source_checkpoint",
            "source checkpoint evidence is missing",
            "run trellara relay after bootstrap to establish a source checkpoint",
        ),
    }
}
