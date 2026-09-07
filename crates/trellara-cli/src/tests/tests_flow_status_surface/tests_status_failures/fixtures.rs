use super::*;

pub(super) fn quarantine_replay_action() -> FlowRecoveryAction {
    FlowRecoveryAction {
        code: "target_quarantine_replay".to_string(),
        reason: "target_postgres_error".to_string(),
        transaction_id: Some("tx-blocked".to_string()),
        commit_lsn: Some("0/16B6D28".to_string()),
        command_templates: vec![
            "trellara quarantine replay-ready --config <config> --transaction-id tx-blocked --commit-lsn 0/16B6D28"
                .to_string(),
        ],
        redelivery_topics: vec!["trellara.source-a.sales.strict".to_string()],
        redelivery_warnings: Vec::new(),
        hint: "seek or redeliver the strict transaction message".to_string(),
    }
}

pub(super) fn source_schema_handoff_action() -> FlowRecoveryAction {
    FlowRecoveryAction {
        code: "source_schema_handoff_required".to_string(),
        reason: "configured source schema fingerprint no longer matches live pgoutput metadata"
            .to_string(),
        transaction_id: None,
        commit_lsn: None,
        command_templates: vec![
            "trellara schema-discover --config <config>".to_string(),
            "trellara contract-test --config <config>".to_string(),
        ],
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: "pause CDC and create a fresh handoff".to_string(),
    }
}

pub(super) fn source_slot_reseed_action() -> FlowRecoveryAction {
    FlowRecoveryAction {
        code: "source_slot_recreate_and_reseed".to_string(),
        reason: "replication slot invalidated".to_string(),
        transaction_id: None,
        commit_lsn: None,
        command_templates: vec!["trellara reseed --config <config>".to_string()],
        redelivery_topics: Vec::new(),
        redelivery_warnings: Vec::new(),
        hint: "reseed affected targets".to_string(),
    }
}
