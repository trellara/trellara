use tokio_postgres::{IsolationLevel, NoTls};

use crate::{
    copyable_columns, ensure_copyable_columns, quote_literal, read_source_rows, write_reseed_rows,
    PostgresReseedConfig, PostgresReseedSummary, Result,
};

pub async fn reseed_postgres_table(config: PostgresReseedConfig) -> Result<PostgresReseedSummary> {
    let (mut source, source_connection) =
        tokio_postgres::connect(&config.source_database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = source_connection.await {
            eprintln!("trellara reseed source postgres connection failed: {error}");
        }
    });
    let (target, target_connection) =
        tokio_postgres::connect(&config.target_database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = target_connection.await {
            eprintln!("trellara reseed target postgres connection failed: {error}");
        }
    });

    let source_snapshot_name = config.source_snapshot_name.as_deref();
    let summary = if let Some(snapshot_name) = source_snapshot_name {
        let source_transaction = source
            .build_transaction()
            .isolation_level(IsolationLevel::RepeatableRead)
            .start()
            .await?;
        source_transaction
            .batch_execute(&format!(
                "set transaction snapshot {}",
                quote_literal(snapshot_name)
            ))
            .await?;
        let copied_columns = copyable_columns(
            &source_transaction,
            &config.relation,
            &config.excluded_columns,
            &config.target_owned_columns,
        )
        .await?;
        ensure_copyable_columns(&config, &copied_columns)?;
        let rows = read_source_rows(
            &source_transaction,
            &config.relation,
            &config.primary_key,
            &copied_columns,
            config.row_filter.as_deref(),
        )
        .await?;
        source_transaction.commit().await?;
        write_reseed_rows(config, target, copied_columns, rows).await
    } else {
        let copied_columns = copyable_columns(
            &source,
            &config.relation,
            &config.excluded_columns,
            &config.target_owned_columns,
        )
        .await?;
        ensure_copyable_columns(&config, &copied_columns)?;
        let rows = read_source_rows(
            &source,
            &config.relation,
            &config.primary_key,
            &copied_columns,
            config.row_filter.as_deref(),
        )
        .await?;
        write_reseed_rows(config, target, copied_columns, rows).await
    }?;

    Ok(summary)
}
