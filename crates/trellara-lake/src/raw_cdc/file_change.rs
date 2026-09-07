use std::collections::BTreeMap;

use trellara_protocol::{ChangeRecord, TransactionEnvelope};

use crate::raw_cdc_accumulator::{
    advance_raw_cdc_lsn, checked_raw_cdc_count_add, RawCdcFileAccumulator,
};
use crate::raw_cdc_naming::raw_cdc_table_name;
use crate::{LakeError, LakePlanConfig};

pub(super) fn accumulate_file_change(
    files: &mut BTreeMap<(String, u32), RawCdcFileAccumulator>,
    plan_config: &LakePlanConfig,
    dataset_id: &str,
    envelope: &TransactionEnvelope,
    transaction_id: &str,
    source_bucket: u32,
    change: &ChangeRecord,
) -> Result<String, LakeError> {
    let relation = change.relation.as_ref().ok_or(LakeError::MissingRelation {
        total_order: change.total_order,
    })?;
    let table = plan_config.table_for(relation)?;
    let relation_name = relation.display_name();
    let table_name = raw_cdc_table_name(dataset_id, table);
    let file_key = (table_name.clone(), source_bucket);
    let file = files
        .entry(file_key)
        .or_insert_with(|| RawCdcFileAccumulator {
            table_name,
            relation: relation_name.clone(),
            source_bucket,
            ..RawCdcFileAccumulator::default()
        });
    file.source_ids.insert(envelope.source_id.clone());
    if file.transactions.insert(transaction_id.to_string()) {
        file.checksum_rollup ^= envelope.checksum;
    }
    file.change_count = checked_raw_cdc_count_add(file.change_count, 1, "file.change_count")?;
    file.idempotency_keys.insert(change.idempotency_key.clone());
    advance_raw_cdc_lsn(
        &mut file.min_commit_lsn,
        &mut file.max_commit_lsn,
        &envelope.commit_lsn,
    )?;
    Ok(relation_name)
}

#[cfg(test)]
#[path = "../tests/tests_raw_cdc_file_change.rs"]
mod tests;
