use trellara_protocol::{ChangeRecord, RelationId, TransactionEnvelope};

use crate::commit_operation::commit_operation;
use crate::commit_row::{primary_key_value, project_row};
use crate::{LakeError, LakeMaterialization, LakeTableConfig, LakeWriteKind, LakeWriteOperation};

pub(crate) fn raw_cdc_operation(
    envelope: &TransactionEnvelope,
    relation: &RelationId,
    table: &LakeTableConfig,
    change: &ChangeRecord,
) -> Result<LakeWriteOperation, LakeError> {
    let image = change.after.as_ref().or(change.before.as_ref());
    let record_key = image
        .map(|image| primary_key_value(image, table, change.total_order))
        .transpose()?;

    Ok(commit_operation(
        envelope,
        relation,
        change,
        LakeMaterialization::RawCdc,
        LakeWriteKind::AppendEvent,
        record_key,
        image
            .map(|image| project_row(image, table))
            .transpose()?
            .unwrap_or_default(),
    ))
}
