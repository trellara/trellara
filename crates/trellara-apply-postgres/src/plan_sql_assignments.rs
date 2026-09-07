use crate::sql::{placeholder_position, quote_ident, SqlValue};
use crate::Result;

pub(crate) fn assignments(values: &[SqlValue]) -> Result<String> {
    assignments_from(values, 1)
}

fn assignments_from(values: &[SqlValue], first_placeholder: usize) -> Result<String> {
    Ok(values
        .iter()
        .enumerate()
        .map(|(index, value)| -> Result<String> {
            Ok(format!(
                "{} = ${}",
                quote_ident(&value.column),
                placeholder_position(first_placeholder, index)?
            ))
        })
        .collect::<Result<Vec<_>>>()?
        .join(", "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SqlValue, SqlValueData};

    #[test]
    fn assignments_number_placeholders_from_one() {
        let values = vec![
            SqlValue {
                column: "amount_cents".to_string(),
                value: SqlValueData::Text("100".to_string()),
            },
            SqlValue {
                column: "updated_at".to_string(),
                value: SqlValueData::Text("2026-08-21".to_string()),
            },
        ];

        assert_eq!(
            assignments(&values).expect("assignments"),
            "\"amount_cents\" = $1, \"updated_at\" = $2"
        );
    }

    #[test]
    fn assignments_quote_identifiers() {
        let values = vec![SqlValue {
            column: "order".to_string(),
            value: SqlValueData::Text("1".to_string()),
        }];

        assert_eq!(assignments(&values).expect("assignments"), "\"order\" = $1");
    }

    #[test]
    fn assignments_reject_placeholder_overflow() {
        let values = vec![
            SqlValue {
                column: "amount_cents".to_string(),
                value: SqlValueData::Text("100".to_string()),
            },
            SqlValue {
                column: "updated_at".to_string(),
                value: SqlValueData::Text("2026-08-21".to_string()),
            },
        ];

        assert!(matches!(
            assignments_from(&values, usize::MAX),
            Err(crate::ApplyError::PlaceholderRangeOverflow {
                start: usize::MAX,
                count: 1,
            })
        ));
    }
}
