use trellara_pg_capture::TableSelector;

use crate::{default_primary_key, parse_init_table, Result, SourceSafetyArgs};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SourceSafetyInitTable {
    pub(crate) selector: TableSelector,
    pub(crate) primary_key: String,
    pub(crate) source_schema_fingerprint: Option<u64>,
}

pub(crate) fn source_safety_init_tables(
    args: &SourceSafetyArgs,
    tables: &[trellara_pg_capture::TablePreflight],
) -> Result<Vec<SourceSafetyInitTable>> {
    let existing = tables
        .iter()
        .filter(|table| table.exists)
        .map(|table| SourceSafetyInitTable {
            selector: TableSelector::new(&table.schema, &table.name),
            primary_key: table
                .primary_key_columns
                .first()
                .cloned()
                .unwrap_or_else(default_primary_key),
            source_schema_fingerprint: table.schema_fingerprint,
        })
        .collect::<Vec<_>>();
    if !existing.is_empty() {
        return Ok(existing);
    }

    args.table
        .iter()
        .map(|table| {
            Ok(SourceSafetyInitTable {
                selector: parse_init_table(table)?,
                primary_key: default_primary_key(),
                source_schema_fingerprint: None,
            })
        })
        .collect()
}
