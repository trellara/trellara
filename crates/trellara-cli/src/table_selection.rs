use crate::{CliError, Result, TableConfig};

pub(crate) fn selected_configured_tables<'a>(
    tables: &'a [TableConfig],
    table_filter: Option<&str>,
    flag_name: &str,
) -> Result<Vec<&'a TableConfig>> {
    let Some(table_filter) = table_filter
        .map(str::trim)
        .filter(|filter| !filter.is_empty())
    else {
        return Ok(tables.iter().collect());
    };

    let selected = tables
        .iter()
        .filter(|table| table.relation_id().display_name() == table_filter)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        let configured = tables
            .iter()
            .map(|table| table.relation_id().display_name())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(CliError::InvalidConfig(format!(
            "{flag_name} {table_filter} is not configured in dataset.tables; configured tables: {configured}"
        )));
    }

    Ok(selected)
}
