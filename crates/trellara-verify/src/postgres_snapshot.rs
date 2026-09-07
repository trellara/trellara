use tokio_postgres::NoTls;

use crate::{
    snapshot_select_sql, PostgresSnapshotConfig, Result, RowSnapshot, TableSnapshot, VerifyError,
};

pub async fn snapshot_postgres_table(config: PostgresSnapshotConfig) -> Result<TableSnapshot> {
    let (client, connection) = tokio_postgres::connect(&config.database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("trellara verify postgres connection failed: {error}");
        }
    });

    let query = snapshot_select_sql(
        &config.relation,
        &config.primary_key,
        config.row_filter.as_deref(),
    );
    let rows = client
        .query(&query, &[&config.excluded_columns])
        .await?
        .into_iter()
        .map(|row| {
            let primary_key: String = row.get("trellara_primary_key");
            let row_json: String = row.get("trellara_row");
            let value =
                serde_json::from_str(&row_json).map_err(|source| VerifyError::ParseRowJson {
                    relation: config.relation.display_name(),
                    source,
                })?;
            RowSnapshot::from_json_value(primary_key, &value).ok_or_else(|| {
                VerifyError::RowNotObject {
                    relation: config.relation.display_name(),
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(TableSnapshot {
        relation: config.relation,
        watermark_lsn: config.watermark_lsn,
        rows,
    })
}
