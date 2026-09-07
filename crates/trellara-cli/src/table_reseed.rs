use trellara_checkpoint::{PostgresCheckpointStore, ReseedEvent, SnapshotHandoffEvent};
use trellara_verify::{reseed_postgres_table, PostgresReseedConfig};

use crate::{
    selected_configured_tables, u64_to_i64_count, usize_to_i64_count, ReseedSummary, Result,
    TableReseedSummary, TrellaraConfig,
};

pub(crate) async fn reseed_configured_tables(
    config: &TrellaraConfig,
    target_database_url: &str,
    table_filter: Option<&str>,
) -> Result<ReseedSummary> {
    let source_store = PostgresCheckpointStore::connect(&config.source.database_url, true).await?;
    let source_checkpoint = trellara_relay::load_source_checkpoint(
        &source_store,
        &config.source.id,
        &config.dataset.id,
    )
    .await?;
    let watermark_lsn = source_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.last_durable_lsn.clone())
        .unwrap_or_default();

    let mut tables = Vec::new();
    let selected_tables =
        selected_configured_tables(&config.dataset.tables, table_filter, "reseed --table")?;

    for table in selected_tables {
        let verify = table.verify.as_ref().cloned().unwrap_or_default();
        let contract = table.contract.as_ref().cloned().unwrap_or_default();
        let row_filter = verify.row_filter.clone();
        tables.push(TableReseedSummary::from_summary(
            reseed_postgres_table(PostgresReseedConfig {
                source_database_url: config.source.database_url.expose().to_string(),
                target_database_url: target_database_url.to_string(),
                relation: table.relation_id(),
                primary_key: verify.primary_key,
                excluded_columns: verify.excluded_columns,
                target_owned_columns: contract.target_owned_columns,
                row_filter: row_filter.clone(),
                watermark_lsn: watermark_lsn.clone(),
                source_snapshot_name: None,
            })
            .await?,
            row_filter,
        ));
    }

    let summary = ReseedSummary {
        source_watermark_lsn: watermark_lsn,
        table_count: tables.len(),
        copied_rows: tables.iter().map(|table| table.copied_rows).sum(),
        tables,
    };
    let target_store = PostgresCheckpointStore::connect(target_database_url, true).await?;
    target_store
        .record_reseed_event(ReseedEvent {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            watermark_lsn: summary.source_watermark_lsn.clone(),
            table_count: usize_to_i64_count("reseed table_count", summary.table_count)?,
            copied_rows: u64_to_i64_count("reseed copied_rows", summary.copied_rows)?,
            completed_at: String::new(),
        })
        .await?;
    for table in &summary.tables {
        target_store
            .record_snapshot_handoff_event(SnapshotHandoffEvent {
                source_id: config.source.id.clone(),
                dataset_id: config.dataset.id.clone(),
                relation: table.relation.clone(),
                watermark_lsn: table.watermark_lsn.clone(),
                copied_rows: u64_to_i64_count("reseed table copied_rows", table.copied_rows)?,
                completed_at: String::new(),
            })
            .await?;
    }

    Ok(summary)
}
