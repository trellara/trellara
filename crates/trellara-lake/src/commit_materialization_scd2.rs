use trellara_protocol::{ChangeRecord, Operation, RelationId, TransactionEnvelope};

use crate::commit_materialization_operation::validate_change_operation;
use crate::commit_operation::commit_operation;
use crate::commit_row::{primary_key_value, project_row, required_image};
use crate::{LakeError, LakeMaterialization, LakeTableConfig, LakeWriteKind, LakeWriteOperation};

pub(crate) fn scd2_operations(
    envelope: &TransactionEnvelope,
    relation: &RelationId,
    table: &LakeTableConfig,
    change: &ChangeRecord,
) -> Result<Vec<LakeWriteOperation>, LakeError> {
    let operation = validate_change_operation(change)?;
    let mut operations = Vec::new();

    match operation {
        Operation::Insert => operations.push(insert_version(envelope, relation, table, change)?),
        Operation::Update => {
            let before =
                required_image(change.before.as_ref(), change.total_order, "scd2_history")?;
            let after = required_image(change.after.as_ref(), change.total_order, "scd2_history")?;
            operations.push(commit_operation(
                envelope,
                relation,
                change,
                LakeMaterialization::Scd2History,
                LakeWriteKind::CloseVersion,
                Some(primary_key_value(before, table, change.total_order)?),
                project_row(before, table)?,
            ));
            operations.push(commit_operation(
                envelope,
                relation,
                change,
                LakeMaterialization::Scd2History,
                LakeWriteKind::InsertVersion,
                Some(primary_key_value(after, table, change.total_order)?),
                project_row(after, table)?,
            ));
        }
        Operation::Delete => {
            let image = required_image(change.before.as_ref(), change.total_order, "scd2_history")?;
            operations.push(commit_operation(
                envelope,
                relation,
                change,
                LakeMaterialization::Scd2History,
                LakeWriteKind::CloseVersion,
                Some(primary_key_value(image, table, change.total_order)?),
                project_row(image, table)?,
            ));
        }
        Operation::Truncate => operations.push(commit_operation(
            envelope,
            relation,
            change,
            LakeMaterialization::Scd2History,
            LakeWriteKind::TruncateHistory,
            None,
            Vec::new(),
        )),
        Operation::Unspecified => unreachable!("validated before materialization"),
    }

    Ok(operations)
}

fn insert_version(
    envelope: &TransactionEnvelope,
    relation: &RelationId,
    table: &LakeTableConfig,
    change: &ChangeRecord,
) -> Result<LakeWriteOperation, LakeError> {
    let image = required_image(change.after.as_ref(), change.total_order, "scd2_history")?;
    Ok(commit_operation(
        envelope,
        relation,
        change,
        LakeMaterialization::Scd2History,
        LakeWriteKind::InsertVersion,
        Some(primary_key_value(image, table, change.total_order)?),
        project_row(image, table)?,
    ))
}
