use trellara_checkpoint::{ApplyQuarantine, TransactionKey};
use trellara_pg_capture::ReplicationSlotStatus;

use crate::{
    quarantine_recovery_hint, replay_redelivery_plan, source_schema_drift_recovery_action,
    source_slot_failover_recovery_actions, source_slot_recovery_actions, CliError,
    FlowRecoveryAction, FlowSchemaDriftSummary, Result, TrellaraConfig,
};

pub(crate) fn require_quarantine_replay_record(
    transaction: &TransactionKey,
    quarantine: Option<ApplyQuarantine>,
) -> Result<ApplyQuarantine> {
    quarantine.ok_or_else(|| {
        CliError::InvalidConfig(format!(
            "quarantine replay-ready requires an existing quarantined transaction for source_id={} dataset_id={} transaction_id={} commit_lsn={}; run trellara quarantine list --config <config> and use the exact transaction boundary before redelivery",
            transaction.source_id,
            transaction.dataset_id,
            transaction.transaction_id,
            transaction.commit_lsn
        ))
    })
}

pub(crate) fn flow_recovery_actions(
    config: &TrellaraConfig,
    source_slot: &ReplicationSlotStatus,
    source_schema_drift: Option<&FlowSchemaDriftSummary>,
    latest_quarantine: Option<&ApplyQuarantine>,
) -> Result<Vec<FlowRecoveryAction>> {
    let mut actions = Vec::new();

    actions.extend(source_slot_recovery_actions(config, source_slot));
    actions.extend(source_slot_failover_recovery_actions(source_slot));

    if let Some(drift) = source_schema_drift {
        actions.push(source_schema_drift_recovery_action(config, drift));
    }

    if let Some(quarantine) = latest_quarantine {
        let redelivery_topics = config.replay_redelivery_topics()?;
        let redelivery_plan = replay_redelivery_plan(
            config,
            &redelivery_topics,
            Some((
                quarantine.transaction_id.as_str(),
                quarantine.commit_lsn.as_str(),
            )),
        )?;
        let redelivery_hint = quarantine_recovery_hint(config, quarantine, &redelivery_topics);
        let mut command_templates = vec![format!(
            "trellara quarantine replay-ready --config <config> --transaction-id {} --commit-lsn {}",
            quarantine.transaction_id, quarantine.commit_lsn
        )];
        command_templates.extend(redelivery_plan.commands);
        actions.push(FlowRecoveryAction {
            code: "target_quarantine_replay".to_string(),
            reason: quarantine.reason.clone(),
            transaction_id: Some(quarantine.transaction_id.clone()),
            commit_lsn: Some(quarantine.commit_lsn.clone()),
            command_templates,
            redelivery_topics,
            redelivery_warnings: redelivery_plan.warnings,
            hint: redelivery_hint,
        });
    }

    Ok(actions)
}
