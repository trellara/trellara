use trellara_protocol::{ChangeRecord, Operation, RelationId, TransactionEnvelope};

use crate::commit_materialization_operation::validate_change_operation;
use crate::commit_operation::commit_operation;
use crate::commit_row::{primary_key_value, project_row, required_image};
use crate::{LakeError, LakeMaterialization, LakeTableConfig, LakeWriteKind, LakeWriteOperation};

pub(crate) fn current_state_operations(
    envelope: &TransactionEnvelope,
    relation: &RelationId,
    table: &LakeTableConfig,
    change: &ChangeRecord,
) -> Result<Vec<LakeWriteOperation>, LakeError> {
    let operation = validate_change_operation(change)?;
    let (write_kind, image) = match operation {
        Operation::Insert | Operation::Update => (
            LakeWriteKind::UpsertCurrent,
            required_image(change.after.as_ref(), change.total_order, "current_state")?,
        ),
        Operation::Delete => (
            LakeWriteKind::DeleteCurrent,
            required_image(change.before.as_ref(), change.total_order, "current_state")?,
        ),
        Operation::Truncate => {
            return Ok(vec![commit_operation(
                envelope,
                relation,
                change,
                LakeMaterialization::CurrentState,
                LakeWriteKind::TruncateCurrent,
                None,
                Vec::new(),
            )]);
        }
        Operation::Unspecified => unreachable!("validated before materialization"),
    };

    Ok(vec![commit_operation(
        envelope,
        relation,
        change,
        LakeMaterialization::CurrentState,
        write_kind,
        Some(primary_key_value(image, table, change.total_order)?),
        project_row(image, table)?,
    )])
}
