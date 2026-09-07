use std::collections::HashMap;

use tokio_postgres::Client;
use tracing::debug;
use trellara_checkpoint::TransactionKey;
use trellara_protocol::TransactionEnvelope;

use crate::checkpoint::{
    clear_quarantined_transaction, record_applied_transaction, transaction_already_applied,
    upsert_checkpoint, upsert_partition_checkpoints,
};
use crate::checkpoint_evidence::validate_apply_checkpoint_evidence;
use crate::executor::execute_statement;
use crate::{plan_envelope_with_policies, ApplyDecision, ApplyOutcome, ApplyTablePolicy, Result};

pub(crate) async fn apply_envelope_transactionally(
    client: &mut Client,
    table_policies: &HashMap<String, ApplyTablePolicy>,
    envelope: &TransactionEnvelope,
) -> Result<ApplyOutcome> {
    validate_apply_checkpoint_evidence(envelope)?;
    let transaction_key = TransactionKey::try_from_envelope(envelope)?;
    let transaction = client.transaction().await?;

    if transaction_already_applied(&transaction, &transaction_key).await? {
        clear_quarantined_transaction(&transaction, &transaction_key).await?;
        transaction.commit().await?;
        debug!(
            transaction_id = %envelope.transaction_id,
            commit_lsn = %envelope.commit_lsn,
            "skipped duplicate transaction"
        );
        return Ok(ApplyOutcome {
            decision: ApplyDecision::SkippedDuplicate,
            applied_changes: 0,
            commit_lsn: envelope.commit_lsn.clone(),
        });
    }

    let statements = plan_envelope_with_policies(envelope, table_policies)?;
    for statement in &statements {
        execute_statement(&transaction, statement).await?;
    }

    record_applied_transaction(&transaction, &transaction_key).await?;
    upsert_checkpoint(&transaction, envelope).await?;
    upsert_partition_checkpoints(&transaction, envelope).await?;
    clear_quarantined_transaction(&transaction, &transaction_key).await?;
    transaction.commit().await?;

    Ok(ApplyOutcome {
        decision: ApplyDecision::Applied,
        applied_changes: statements.len(),
        commit_lsn: envelope.commit_lsn.clone(),
    })
}
