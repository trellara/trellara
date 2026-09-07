use trellara_protocol::TransactionEnvelope;

use crate::commit_materialization_current::current_state_operations;
use crate::commit_materialization_operation::validate_change_operation;
use crate::commit_materialization_raw::raw_cdc_operation;
use crate::commit_materialization_scd2::scd2_operations;
use crate::commit_visibility::visibility_boundary;
use crate::{LakeCommitPlan, LakeError, LakePlanConfig};

pub fn plan_commit(
    envelope: &TransactionEnvelope,
    config: &LakePlanConfig,
) -> Result<LakeCommitPlan, LakeError> {
    envelope.verify_checksum()?;
    envelope.validate()?;
    let visibility_boundary = visibility_boundary(envelope)?;
    let mut operations = Vec::new();

    for change in &envelope.changes {
        let relation = change.relation.as_ref().ok_or(LakeError::MissingRelation {
            total_order: change.total_order,
        })?;
        let table = config.table_for(relation)?;

        validate_change_operation(change)?;
        operations.push(raw_cdc_operation(envelope, relation, table, change)?);
        operations.extend(current_state_operations(envelope, relation, table, change)?);
        operations.extend(scd2_operations(envelope, relation, table, change)?);
    }

    Ok(LakeCommitPlan {
        source_id: envelope.source_id.clone(),
        dataset_id: envelope.dataset_id.clone(),
        transaction_id: envelope.transaction_id.clone(),
        commit_lsn: envelope.commit_lsn.clone(),
        commit_timestamp_ms: envelope.commit_timestamp_ms,
        visibility_boundary,
        operation_count: operations.len(),
        operations,
    })
}
