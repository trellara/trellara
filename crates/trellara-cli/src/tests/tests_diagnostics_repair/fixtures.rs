use super::*;

pub(super) fn blocked_recovery_action() -> FlowRecoveryAction {
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
        redelivery_warnings: vec![
            "local redelivery boundary is incomplete".to_string(),
            "redelivery requires operator confirmation".to_string(),
        ],
        hint: "seek or redeliver the strict transaction message".to_string(),
    }
}
