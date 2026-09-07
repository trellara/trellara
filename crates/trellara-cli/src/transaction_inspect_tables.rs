use std::collections::BTreeMap;

use trellara_protocol::{ChangeRecord, Operation};

use crate::TransactionInspectTable;

pub(crate) fn transaction_inspect_tables(changes: &[ChangeRecord]) -> Vec<TransactionInspectTable> {
    let mut affected_tables = BTreeMap::<String, TransactionInspectTable>::new();
    for change in changes {
        let relation = change
            .relation
            .as_ref()
            .map(|relation| relation.display_name())
            .unwrap_or_else(|| "unknown".to_string());
        let table =
            affected_tables
                .entry(relation.clone())
                .or_insert_with(|| TransactionInspectTable {
                    relation,
                    ..TransactionInspectTable::default()
                });
        table.event_count += 1;
        match Operation::try_from(change.operation).unwrap_or(Operation::Unspecified) {
            Operation::Insert => table.inserts += 1,
            Operation::Update => table.updates += 1,
            Operation::Delete => table.deletes += 1,
            Operation::Truncate => table.truncates += 1,
            Operation::Unspecified => {}
        }
    }
    affected_tables.into_values().collect()
}
