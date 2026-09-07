use crate::{
    reseed_clear_sql, reseed_insert_sql, CopyableColumn, PostgresReseedConfig,
    PostgresReseedSummary, Result, VerifyError,
};

pub(crate) fn ensure_copyable_columns(
    config: &PostgresReseedConfig,
    copied_columns: &[CopyableColumn],
) -> Result<()> {
    if copied_columns.is_empty() {
        Err(VerifyError::NoCopyableColumns {
            relation: config.relation.display_name(),
        })
    } else {
        Ok(())
    }
}

pub(crate) async fn write_reseed_rows(
    config: PostgresReseedConfig,
    mut target: tokio_postgres::Client,
    copied_columns: Vec<CopyableColumn>,
    rows: Vec<Vec<Option<String>>>,
) -> Result<PostgresReseedSummary> {
    ensure_copyable_columns(&config, &copied_columns)?;
    let transaction = target.transaction().await?;
    transaction
        .batch_execute(&reseed_clear_sql(
            &config.relation,
            config.row_filter.as_deref(),
        ))
        .await?;
    let insert_sql = reseed_insert_sql(&config.relation, &copied_columns);
    for row in &rows {
        let params = row
            .iter()
            .map(|value| value as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect::<Vec<_>>();
        transaction.execute(&insert_sql, &params).await?;
    }
    transaction.commit().await?;

    reseed_summary(config, copied_columns, rows.len())
}

fn reseed_summary(
    config: PostgresReseedConfig,
    copied_columns: Vec<CopyableColumn>,
    row_count: usize,
) -> Result<PostgresReseedSummary> {
    Ok(PostgresReseedSummary {
        relation: config.relation,
        watermark_lsn: config.watermark_lsn,
        copied_rows: reseed_row_count(row_count)?,
        copied_columns: copied_columns
            .into_iter()
            .map(|column| column.name)
            .collect(),
    })
}

fn reseed_row_count(row_count: usize) -> Result<u64> {
    u64::try_from(row_count).map_err(|_| VerifyError::ReseedRowCountOverflow { row_count })
}

#[cfg(test)]
mod tests {
    use trellara_protocol::RelationId;

    use super::*;

    #[test]
    fn reseed_row_count_accepts_normal_counts() {
        assert_eq!(reseed_row_count(42).expect("row count"), 42);
    }

    #[test]
    fn reseed_summary_preserves_watermark_and_copied_column_order() {
        let summary = reseed_summary(
            PostgresReseedConfig {
                source_database_url: "postgres://source".to_string(),
                target_database_url: "postgres://target".to_string(),
                relation: RelationId::new(42, "public", "sales"),
                primary_key: "id".to_string(),
                watermark_lsn: "0/16B9000".to_string(),
                excluded_columns: Vec::new(),
                target_owned_columns: Vec::new(),
                row_filter: None,
                source_snapshot_name: None,
            },
            vec![
                CopyableColumn {
                    name: "id".to_string(),
                    type_name: "text".to_string(),
                },
                CopyableColumn {
                    name: "receipt_bytes".to_string(),
                    type_name: "bytea".to_string(),
                },
            ],
            2,
        )
        .expect("summary");

        assert_eq!(summary.relation, RelationId::new(42, "public", "sales"));
        assert_eq!(summary.watermark_lsn, "0/16B9000");
        assert_eq!(summary.copied_rows, 2);
        assert_eq!(summary.copied_columns, vec!["id", "receipt_bytes"]);
    }
}
