use trellara_pg_capture::ReplicationSlotStatus;

use crate::{
    source_postgres_risk_factors, source_slot_issue_recommendation, strongest_source_factor,
    FlowSchemaDriftSummary, SnapshotHandoffProofStatus,
};

pub(crate) struct ReportActionInputs<'a> {
    pub(crate) source_slot: &'a ReplicationSlotStatus,
    pub(crate) source_subscription_conflicts_safe: bool,
    pub(crate) source_schema_contract_safe: bool,
    pub(crate) source_schema_drift: Option<&'a FlowSchemaDriftSummary>,
    pub(crate) source_wal_retention_safe: bool,
    pub(crate) source_checkpoint_durable: bool,
    pub(crate) target_caught_up: bool,
    pub(crate) source_to_target_lag_bytes: Option<u64>,
    pub(crate) partition_watermark_ready: bool,
    pub(crate) no_target_quarantine: bool,
    pub(crate) snapshot_handoff_status: SnapshotHandoffProofStatus,
    pub(crate) latest_validation_converged: bool,
    pub(crate) latest_validation_current: bool,
}

pub(crate) fn report_recommended_actions(input: ReportActionInputs<'_>) -> Vec<String> {
    let mut actions = Vec::new();
    let source_slot_factor =
        strongest_source_factor(source_postgres_risk_factors(input.source_slot));
    let source_slot_safe = input.source_slot.exists
        && input.source_slot.issues.is_empty()
        && source_slot_factor.is_none();
    if !source_slot_safe {
        if let Some(factor) = source_slot_factor {
            actions.push(factor.recommendation);
        } else if input.source_slot.exists {
            actions.push(source_slot_issue_recommendation(input.source_slot).to_string());
        } else {
            actions.push(
                "run trellara bootstrap to create or repair the source replication slot"
                    .to_string(),
            );
        }
    }
    if !input.source_subscription_conflicts_safe {
        actions.push(
            "pause affected subscriptions, resolve subscriber-side divergence, then verify or reseed before trusting CDC output"
                .to_string(),
        );
    }
    if !input.source_schema_contract_safe {
        actions.push(source_schema_handoff_action(input.source_schema_drift));
    }
    if !input.source_wal_retention_safe {
        actions.push(
            "drain relay/apply lag or reseed slow targets before source WAL retention grows further"
                .to_string(),
        );
    }
    if !input.source_checkpoint_durable {
        actions.push("run trellara relay until source durable checkpoint catches up".to_string());
    }
    if !input.target_caught_up {
        actions.push(target_catch_up_action(input.source_to_target_lag_bytes));
    }
    if !input.partition_watermark_ready {
        actions.push(
            "run the barrier-aware applier for all partition topics until partition watermarks are complete and caught up"
                .to_string(),
        );
    }
    if !input.no_target_quarantine {
        actions.push("run trellara quarantine list, repair the target contract, then trellara quarantine replay-ready before redelivery".to_string());
    }
    if !input.snapshot_handoff_status.is_verified() {
        actions.push(input.snapshot_handoff_status.recommendation().to_string());
    }
    if !input.latest_validation_converged {
        actions.push(
            "run trellara verify after repair; use trellara reseed if checksum drift remains"
                .to_string(),
        );
    } else if !input.latest_validation_current {
        actions.push(
            "run trellara verify until validation watermarks reach the current source and target checkpoints"
                .to_string(),
        );
    }
    actions
}

fn target_catch_up_action(source_to_target_lag_bytes: Option<u64>) -> String {
    if source_to_target_lag_bytes.is_some_and(|lag| lag > 0) {
        return "run trellara relay and trellara apply until target applied watermark reaches the source durable watermark".to_string();
    }
    "run trellara apply until target applied watermark catches up".to_string()
}

fn source_schema_handoff_action(drift: Option<&FlowSchemaDriftSummary>) -> String {
    let relation_scope = drift
        .filter(|drift| !drift.relations.is_empty())
        .map(|drift| format!(" for {}", drift.relations.join(", ")))
        .unwrap_or_default();
    format!(
        "pause CDC{relation_scope}, run trellara schema-discover and trellara contract-test, then create a fresh snapshot-to-stream handoff before resuming"
    )
}
