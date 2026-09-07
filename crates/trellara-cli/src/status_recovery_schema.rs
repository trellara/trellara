use crate::{FlowRecoveryAction, FlowSchemaDriftSummary, TrellaraConfig};

pub(crate) fn source_schema_drift_recovery_action(
    config: &TrellaraConfig,
    drift: &FlowSchemaDriftSummary,
) -> FlowRecoveryAction {
    let mut command_templates = vec![
        "trellara schema-discover --config <config>".to_string(),
        "trellara contract-test --config <config>".to_string(),
        "trellara snapshot --config <config> --force".to_string(),
        "trellara relay --config <config>".to_string(),
    ];
    if config.target.is_some() {
        command_templates.push("trellara apply --config <config>".to_string());
        command_templates.push("trellara verify --config <config>".to_string());
    }
    FlowRecoveryAction {
        code: "source_schema_handoff_required".to_string(),
        reason: format!("{} for {}", drift.reason, drift.relations.join(", ")),
        transaction_id: None,
        commit_lsn: None,
        command_templates,
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: "pause CDC for the affected flow, discover the new schema fingerprint, review and update the contract, then create a fresh audited snapshot-to-stream handoff before resuming"
            .to_string(),
    }
}
