use trellara_pg_capture::ReplicationSlotStatus;

use crate::{
    source_slot_failover_factors, source_slot_issue_factors, source_slot_wal_boundary_broken,
    source_slot_wal_pressure_active, source_wal_retention_factor, FlowRecoveryAction,
    TrellaraConfig,
};

pub(crate) fn source_slot_recovery_actions(
    config: &TrellaraConfig,
    source_slot: &ReplicationSlotStatus,
) -> Vec<FlowRecoveryAction> {
    if !source_slot.exists {
        return vec![source_slot_bootstrap_or_reseed_action(config, source_slot)];
    }
    if source_slot_wal_boundary_broken(source_slot) {
        return vec![source_slot_recreate_and_reseed_action(config, source_slot)];
    }
    if source_slot_wal_pressure_active(source_slot) {
        return vec![source_slot_drain_before_wal_loss_action(
            config,
            source_slot,
        )];
    }

    Vec::new()
}

pub(crate) fn source_slot_failover_recovery_actions(
    source_slot: &ReplicationSlotStatus,
) -> Vec<FlowRecoveryAction> {
    source_slot_failover_factors(source_slot)
        .into_iter()
        .map(|factor| FlowRecoveryAction {
            code: match factor.code.as_str() {
                "source_slot_failover_disabled" => "source_slot_enable_failover_slot",
                "source_slot_failover_not_synced" => "source_slot_wait_for_failover_sync",
                _ => "source_slot_failover_prepare",
            }
            .to_string(),
            reason: factor.evidence,
            transaction_id: None,
            commit_lsn: None,
            command_templates: vec![
                "trellara check --config <config>".to_string(),
                "trellara status --config <config> --view alerts".to_string(),
            ],
            redelivery_topics: Vec::new(),
            redelivery_warnings: Vec::new(),
            hint: factor.recommendation,
        })
        .collect()
}

fn source_slot_bootstrap_or_reseed_action(
    config: &TrellaraConfig,
    source_slot: &ReplicationSlotStatus,
) -> FlowRecoveryAction {
    let mut command_templates = vec![
        "trellara bootstrap --config <config>".to_string(),
        "trellara status --config <config> --view alerts".to_string(),
    ];
    if config.target.is_some() {
        command_templates.insert(1, "trellara snapshot --config <config> --force".to_string());
        command_templates.insert(2, "trellara relay --config <config>".to_string());
        command_templates.insert(3, "trellara apply --config <config>".to_string());
        command_templates.push("trellara verify --config <config>".to_string());
    }
    FlowRecoveryAction {
        code: "source_slot_bootstrap_or_reseed".to_string(),
        reason: source_slot
            .issues
            .first()
            .cloned()
            .unwrap_or_else(|| "source replication slot is missing".to_string()),
        transaction_id: None,
        commit_lsn: None,
        command_templates,
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: "source slot is missing; create or repair the slot, and for existing targets use a fresh snapshot-to-stream handoff before resuming CDC"
            .to_string(),
    }
}

fn source_slot_recreate_and_reseed_action(
    config: &TrellaraConfig,
    source_slot: &ReplicationSlotStatus,
) -> FlowRecoveryAction {
    let mut command_templates = vec![
        "trellara bootstrap --config <config>".to_string(),
        "trellara relay --config <config>".to_string(),
    ];
    if config.target.is_some() {
        command_templates.insert(1, "trellara reseed --config <config>".to_string());
        command_templates.push("trellara apply --config <config>".to_string());
        command_templates.push("trellara verify --config <config>".to_string());
    }
    FlowRecoveryAction {
        code: "source_slot_recreate_and_reseed".to_string(),
        reason: source_slot
            .issues
            .first()
            .cloned()
            .unwrap_or_else(|| "source replication slot WAL boundary is broken".to_string()),
        transaction_id: None,
        commit_lsn: None,
        command_templates,
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: "source WAL needed by the old slot is no longer available; recreate the slot, reseed affected targets from a fresh snapshot handoff, then resume CDC"
            .to_string(),
    }
}

fn source_slot_drain_before_wal_loss_action(
    config: &TrellaraConfig,
    source_slot: &ReplicationSlotStatus,
) -> FlowRecoveryAction {
    let mut command_templates = vec![
        "trellara relay --config <config>".to_string(),
        "trellara status --config <config> --view alerts".to_string(),
    ];
    if config.target.is_some() {
        command_templates.insert(1, "trellara apply --config <config>".to_string());
        command_templates.push("trellara verify --config <config>".to_string());
    }
    FlowRecoveryAction {
        code: "source_slot_drain_before_wal_loss".to_string(),
        reason: source_slot_issue_factors(source_slot)
            .into_iter()
            .find(|factor| {
                matches!(
                    factor.code.as_str(),
                    "source_slot_wal_unreserved" | "source_slot_safe_wal_exhausted"
                )
            })
            .map(|factor| factor.evidence)
            .or_else(|| {
                source_wal_retention_factor(
                    source_slot,
                    None,
                    "drain relay/apply lag or reseed slow targets before source WAL retention grows further",
                )
                .map(|factor| factor.evidence)
            })
            .or_else(|| source_slot.issues.first().cloned())
            .unwrap_or_else(|| "source replication slot WAL retention is at risk".to_string()),
        transaction_id: None,
        commit_lsn: None,
        command_templates,
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: "source WAL is still recoverable but at risk; drain relay/apply lag now, then verify before the slot crosses into mandatory reseed"
            .to_string(),
    }
}
