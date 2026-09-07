use std::collections::HashSet;

use tokio_postgres::GenericClient;
use trellara_protocol::RelationId;

use crate::{normalized_row_filter, quote_ident, quote_table, CopyableColumn, Result};

pub(crate) async fn copyable_columns(
    client: &(impl GenericClient + Sync),
    relation: &RelationId,
    excluded_columns: &[String],
    target_owned_columns: &[String],
) -> Result<Vec<CopyableColumn>> {
    let excluded = excluded_columns
        .iter()
        .chain(target_owned_columns)
        .cloned()
        .collect::<HashSet<_>>();
    Ok(client
        .query(
            r#"
            select column_name
                   , udt_name
              from information_schema.columns
             where table_schema = $1
               and table_name = $2
             order by ordinal_position
            "#,
            &[&relation.schema, &relation.table],
        )
        .await?
        .into_iter()
        .map(|row| CopyableColumn {
            name: row.get(0),
            type_name: row.get(1),
        })
        .filter(|column| !excluded.contains(&column.name))
        .collect())
}

pub(crate) async fn read_source_rows(
    client: &(impl GenericClient + Sync),
    relation: &RelationId,
    primary_key: &str,
    columns: &[CopyableColumn],
    row_filter: Option<&str>,
) -> Result<Vec<Vec<Option<String>>>> {
    let query = source_reseed_select_sql(relation, primary_key, columns, row_filter);

    Ok(client
        .query(&query, &[])
        .await?
        .into_iter()
        .map(|row| {
            (0..columns.len())
                .map(|index| row.get::<_, Option<String>>(index))
                .collect::<Vec<_>>()
        })
        .collect())
}

pub(crate) fn source_reseed_select_sql(
    relation: &RelationId,
    primary_key: &str,
    columns: &[CopyableColumn],
    row_filter: Option<&str>,
) -> String {
    let selected_columns = columns
        .iter()
        .map(|column| {
            if column.is_bytea() {
                format!("encode({}, 'hex')", quote_ident(&column.name))
            } else {
                format!("{}::text", quote_ident(&column.name))
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    let table = quote_table(relation);
    let primary_key = quote_ident(primary_key);
    if let Some(row_filter) = normalized_row_filter(row_filter) {
        format!("select {selected_columns} from {table} where ({row_filter}) order by {primary_key}::text")
    } else {
        format!("select {selected_columns} from {table} order by {primary_key}::text")
    }
}
