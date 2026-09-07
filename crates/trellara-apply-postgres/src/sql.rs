use trellara_protocol::{ColumnValue, RelationId, ValueKind};

use crate::{ApplyError, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlStatement {
    pub sql: String,
    pub values: Vec<SqlValue>,
    pub requires_row_match: bool,
    pub total_order: u32,
    pub operation: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SqlValue {
    pub column: String,
    pub value: SqlValueData,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SqlValueData {
    Null,
    Text(String),
    Binary(Vec<u8>),
}

impl SqlValue {
    pub(crate) fn from_column(column: &ColumnValue) -> Result<Self> {
        let value_kind = ValueKind::try_from(column.value_kind).unwrap_or(ValueKind::Unspecified);
        let value = match value_kind {
            ValueKind::Null => SqlValueData::Null,
            ValueKind::Text => SqlValueData::Text(column.text_value.clone()),
            ValueKind::Binary => SqlValueData::Binary(column.binary_value.to_vec()),
            ValueKind::Unspecified | ValueKind::UnchangedToast => {
                return Err(ApplyError::UnsupportedValueKind {
                    column: column.name.clone(),
                    value_kind: column.value_kind,
                });
            }
        };

        Ok(Self {
            column: column.name.clone(),
            value,
        })
    }
}

pub(crate) fn placeholders(start: usize, count: usize) -> Result<Vec<String>> {
    let end = placeholder_range_end(start, count)?;
    Ok((start..end)
        .map(|position| format!("${position}"))
        .collect())
}

pub(crate) fn placeholder_position(start: usize, offset: usize) -> Result<usize> {
    start
        .checked_add(offset)
        .ok_or(ApplyError::PlaceholderRangeOverflow {
            start,
            count: offset,
        })
}

fn placeholder_range_end(start: usize, count: usize) -> Result<usize> {
    start
        .checked_add(count)
        .ok_or(ApplyError::PlaceholderRangeOverflow { start, count })
}

pub(crate) fn qualified_table(relation: &RelationId) -> String {
    format!(
        "{}.{}",
        quote_ident(&relation.schema),
        quote_ident(&relation.table)
    )
}

pub(crate) fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_ident_escapes_embedded_quotes() {
        assert_eq!(quote_ident("sales\"archive"), "\"sales\"\"archive\"");
    }

    #[test]
    fn placeholders_use_postgres_one_based_positions() {
        assert_eq!(
            placeholders(3, 3).expect("placeholders"),
            vec!["$3", "$4", "$5"]
        );
        assert!(placeholders(1, 0).expect("empty placeholders").is_empty());
    }

    #[test]
    fn placeholders_reject_range_overflow() {
        assert!(matches!(
            placeholders(usize::MAX, 1),
            Err(ApplyError::PlaceholderRangeOverflow {
                start: usize::MAX,
                count: 1,
            })
        ));
    }

    #[test]
    fn placeholder_position_rejects_offset_overflow() {
        assert!(matches!(
            placeholder_position(usize::MAX, 1),
            Err(ApplyError::PlaceholderRangeOverflow {
                start: usize::MAX,
                count: 1,
            })
        ));
    }

    #[test]
    fn qualified_table_quotes_schema_and_table_independently() {
        let relation = RelationId::new(42, "tenant\"a", "order");

        assert_eq!(qualified_table(&relation), "\"tenant\"\"a\".\"order\"");
    }
}
