use crate::{selected_configured_tables, Result, TableConfig};

pub(crate) struct SnapshotCopyPlan<'a> {
    pub(crate) selected_tables: Vec<&'a TableConfig>,
    pub(crate) handoff_relations: Vec<String>,
}

impl<'a> SnapshotCopyPlan<'a> {
    pub(crate) fn from_tables(
        tables: &'a [TableConfig],
        table_filter: Option<&str>,
    ) -> Result<Self> {
        let selected_tables = selected_configured_tables(tables, table_filter, "snapshot --table")?;
        let handoff_relations = tables
            .iter()
            .map(|table| table.relation_id().display_name())
            .collect();

        Ok(Self {
            selected_tables,
            handoff_relations,
        })
    }
}
