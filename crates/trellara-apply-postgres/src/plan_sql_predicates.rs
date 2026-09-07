use crate::sql::{placeholder_position, quote_ident, SqlValue};
use crate::Result;

pub(crate) fn key_predicate(values: &[SqlValue], first_placeholder: usize) -> Result<String> {
    Ok(values
        .iter()
        .enumerate()
        .map(|(index, value)| -> Result<String> {
            Ok(format!(
                "{} is not distinct from ${}",
                quote_ident(&value.column),
                placeholder_position(first_placeholder, index)?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(" and "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SqlValue, SqlValueData};

    #[test]
    fn key_predicate_starts_after_set_values() {
        let values = vec![
            SqlValue {
                column: "store_id".to_string(),
                value: SqlValueData::Text("store-001".to_string()),
            },
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-1".to_string()),
            },
        ];

        assert_eq!(
            key_predicate(&values, 3).expect("key predicate"),
            "\"store_id\" is not distinct from $3 and \"id\" is not distinct from $4"
        );
    }

    #[test]
    fn key_predicate_quotes_identifiers() {
        let values = vec![SqlValue {
            column: "order".to_string(),
            value: SqlValueData::Text("1".to_string()),
        }];

        assert_eq!(
            key_predicate(&values, 1).expect("key predicate"),
            "\"order\" is not distinct from $1"
        );
    }

    #[test]
    fn key_predicate_rejects_placeholder_overflow() {
        let values = vec![
            SqlValue {
                column: "store_id".to_string(),
                value: SqlValueData::Text("store-001".to_string()),
            },
            SqlValue {
                column: "id".to_string(),
                value: SqlValueData::Text("sale-1".to_string()),
            },
        ];

        assert!(matches!(
            key_predicate(&values, usize::MAX),
            Err(crate::ApplyError::PlaceholderRangeOverflow {
                start: usize::MAX,
                count: 1,
            })
        ));
    }
}
