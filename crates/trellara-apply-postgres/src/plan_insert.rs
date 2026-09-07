use trellara_protocol::ChangeRecord;

use crate::plan_row::{is_target_owned, relation, row_image};
use crate::sql::{placeholders, qualified_table, quote_ident, SqlStatement, SqlValue};
use crate::{ApplyTablePolicy, Result};

pub(crate) fn plan_insert(
    change: &ChangeRecord,
    policy: Option<&ApplyTablePolicy>,
) -> Result<SqlStatement> {
    let relation = relation(change)?;
    let after = row_image(change, "after", change.after.as_ref())?;
    let values = after
        .columns
        .iter()
        .filter(|column| !is_target_owned(policy, &column.name))
        .map(SqlValue::from_column)
        .collect::<Result<Vec<_>>>()?;
    if values.is_empty() {
        return Ok(SqlStatement {
            sql: format!("insert into {} default values", qualified_table(relation)),
            values,
            requires_row_match: false,
            total_order: change.total_order,
            operation: "insert",
        });
    }
    let columns = values
        .iter()
        .map(|value| quote_ident(&value.column))
        .collect::<Vec<_>>()
        .join(", ");
    let placeholders = placeholders(1, values.len())?.join(", ");

    Ok(SqlStatement {
        sql: format!(
            "insert into {} ({}) values ({})",
            qualified_table(relation),
            columns,
            placeholders
        ),
        values,
        requires_row_match: false,
        total_order: change.total_order,
        operation: "insert",
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellara_protocol::{
        idempotency_key, ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity,
        RowImage,
    };

    #[test]
    fn insert_planner_filters_target_owned_columns() {
        let policy = ApplyTablePolicy {
            relation: relation_id(),
            target_owned_columns: vec!["updated_at".to_string()],
        };

        let statement = plan_insert(&insert_change(), Some(&policy)).expect("statement");

        assert_eq!(
            statement.sql,
            "insert into \"public\".\"sales\" (\"id\", \"amount_cents\") values ($1, $2)"
        );
        assert_eq!(
            statement
                .values
                .iter()
                .map(|value| value.column.as_str())
                .collect::<Vec<_>>(),
            vec!["id", "amount_cents"]
        );
        assert!(!statement.requires_row_match);
    }

    #[test]
    fn insert_planner_uses_default_values_when_all_columns_are_target_owned() {
        let policy = ApplyTablePolicy {
            relation: relation_id(),
            target_owned_columns: vec![
                "id".to_string(),
                "amount_cents".to_string(),
                "updated_at".to_string(),
            ],
        };

        let statement = plan_insert(&insert_change(), Some(&policy)).expect("statement");

        assert_eq!(
            statement.sql,
            "insert into \"public\".\"sales\" default values"
        );
        assert!(statement.values.is_empty());
        assert_eq!(statement.operation, "insert");
    }

    fn insert_change() -> ChangeRecord {
        ChangeRecord {
            transaction_id: "tx-1".to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(relation_id()),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            after: Some(RowImage::new(vec![
                ColumnValue::text("id", 23, "sale-1", true),
                ColumnValue::text("amount_cents", 20, "1299", false),
                ColumnValue::text("updated_at", 25, "2026-08-21", false),
            ])),
            idempotency_key: idempotency_key("source", "0/16B6C50", "tx-1", 1),
        }
    }

    fn relation_id() -> RelationId {
        RelationId::new(42, "public", "sales")
    }
}
