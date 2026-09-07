use trellara_protocol::RelationId;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopyableColumn {
    pub(crate) name: String,
    pub(crate) type_name: String,
}

impl CopyableColumn {
    pub(crate) fn is_bytea(&self) -> bool {
        self.type_name == "bytea"
    }
}

pub(crate) fn snapshot_select_sql(
    relation: &RelationId,
    primary_key: &str,
    row_filter: Option<&str>,
) -> String {
    let mut query = format!(
        "select {primary_key}::text as trellara_primary_key, \
         (to_jsonb(t) - $1::text[])::text as trellara_row \
         from {table} as t \
         order by {primary_key}::text",
        primary_key = quote_ident(primary_key),
        table = quote_table(relation),
    );
    if let Some(row_filter) = normalized_row_filter(row_filter) {
        query = format!(
            "select {primary_key}::text as trellara_primary_key, \
             (to_jsonb(t) - $1::text[])::text as trellara_row \
             from {table} as t \
             where ({row_filter}) \
             order by {primary_key}::text",
            primary_key = quote_ident(primary_key),
            table = quote_table(relation),
        );
    }
    query
}

pub(crate) fn reseed_clear_sql(relation: &RelationId, row_filter: Option<&str>) -> String {
    match normalized_row_filter(row_filter) {
        Some(row_filter) => format!("delete from {} where ({row_filter})", quote_table(relation)),
        None => format!("truncate table {}", quote_table(relation)),
    }
}

pub(crate) fn normalized_row_filter(row_filter: Option<&str>) -> Option<&str> {
    row_filter
        .map(str::trim)
        .filter(|filter| !filter.is_empty())
}

pub(crate) fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

pub(crate) fn reseed_insert_sql(relation: &RelationId, columns: &[CopyableColumn]) -> String {
    let column_list = columns
        .iter()
        .map(|column| quote_ident(&column.name))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = (1..=columns.len())
        .map(|position| {
            let column = &columns[position - 1];
            if column.is_bytea() {
                format!("decode(${position}, 'hex')")
            } else {
                format!("${position}")
            }
        })
        .collect::<Vec<_>>()
        .join(", ");
    format!(
        "insert into {} ({column_list}) values ({placeholders})",
        quote_table(relation)
    )
}

pub(crate) fn quote_table(relation: &RelationId) -> String {
    format!(
        "{}.{}",
        quote_ident(&relation.schema),
        quote_ident(&relation.table)
    )
}

pub(crate) fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}
