use std::collections::BTreeSet;

use crate::LakeEpochSummary;

pub(crate) fn validate_table_rollup_rows(epoch: &LakeEpochSummary) -> Result<(), String> {
    let mut seen_relations = BTreeSet::new();
    for table in &epoch.table_rollups {
        validate_relation(&table.relation)?;
        if !seen_relations.insert(table.relation.clone()) {
            return Err(format!(
                "table rollup has duplicate relation {}",
                table.relation
            ));
        }
        validate_table_counts(
            &table.relation,
            table.transaction_count,
            table.change_count,
            table.checksum_rollup,
        )?;
    }
    Ok(())
}

fn validate_relation(relation: &str) -> Result<(), String> {
    if relation.trim().is_empty() || relation.trim() != relation {
        Err(format!("table rollup has invalid relation {relation:?}"))
    } else {
        Ok(())
    }
}

fn validate_table_counts(
    relation: &str,
    transaction_count: usize,
    change_count: usize,
    checksum_rollup: u64,
) -> Result<(), String> {
    if change_count == 0 && transaction_count > 0 {
        return Err(format!(
            "table rollup relation {relation} has transactions without changes"
        ));
    }
    if change_count == 0 && checksum_rollup != 0 {
        return Err(format!(
            "table rollup relation {relation} has checksum without changes"
        ));
    }
    if transaction_count == 0 && change_count > 0 {
        return Err(format!(
            "table rollup relation {relation} has changes without transactions"
        ));
    }
    Ok(())
}
