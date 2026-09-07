use trellara_apply_postgres::{
    plan_target_ddl_transaction, target_ddl_apply_plan_from_envelope, ApplyError,
};
use trellara_protocol::TransactionEnvelope;

use crate::Result;

pub(crate) struct DdlEnvelopeTargetPlan {
    pub(crate) executable: bool,
    pub(crate) blockers: Vec<String>,
    pub(crate) target_sql: Vec<String>,
}

pub(crate) fn target_ddl_sql_plan(
    envelope: &TransactionEnvelope,
    target_configured: bool,
) -> Result<DdlEnvelopeTargetPlan> {
    let mut blockers = Vec::new();
    let mut target_sql = Vec::new();

    match target_ddl_apply_plan_from_envelope(envelope)? {
        Some(plan) => match plan_target_ddl_transaction(plan) {
            Ok(transaction_plan) => {
                target_sql = transaction_plan
                    .statements
                    .into_iter()
                    .map(|statement| statement.sql)
                    .collect();
            }
            Err(ApplyError::DdlApplyBlocked {
                blockers: plan_blockers,
                ..
            }) => blockers.extend(plan_blockers),
            Err(ApplyError::UnsafeDdlStatement { change, reason }) => {
                blockers.push(format!("{change} is unsafe for auto-apply: {reason}"));
            }
            Err(error) => return Err(error.into()),
        },
        None => blockers.push("envelope contains no DDL events".to_string()),
    }

    if !target_configured {
        blockers.push("target.database_url is required to execute target DDL".to_string());
    }

    Ok(DdlEnvelopeTargetPlan {
        executable: blockers.is_empty() && !target_sql.is_empty(),
        blockers,
        target_sql,
    })
}
