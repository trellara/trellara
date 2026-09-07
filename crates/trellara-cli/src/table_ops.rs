use trellara_checkpoint::{FlowKey, PostgresCheckpointStore, ValidationEvent};
use trellara_verify::{
    compare_snapshots, ensure_target_caught_up, snapshot_postgres_table, PostgresSnapshotConfig,
};

use crate::{
    selected_configured_tables, usize_to_i64_count, validation_evidence_sha256,
    verified_snapshot_run, ChecksumStatus, Result, TableVerifySummary, TrellaraConfig,
    VerifySummary,
};

pub(crate) async fn verify_configured_tables(
    config: &TrellaraConfig,
    target_database_url: &str,
    table_filter: Option<&str>,
) -> Result<VerifySummary> {
    let source_store = PostgresCheckpointStore::connect(&config.source.database_url, true).await?;
    let target_store = PostgresCheckpointStore::connect(target_database_url, true).await?;
    let source_checkpoint = trellara_relay::load_source_checkpoint(
        &source_store,
        &config.source.id,
        &config.dataset.id,
    )
    .await?;
    let target_checkpoint = trellara_relay::load_source_checkpoint(
        &target_store,
        &config.source.id,
        &config.dataset.id,
    )
    .await?;
    let source_watermark = source_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.last_durable_lsn.clone())
        .unwrap_or_default();
    let target_watermark = target_checkpoint
        .as_ref()
        .map(|checkpoint| checkpoint.last_applied_lsn.clone())
        .unwrap_or_default();
    ensure_target_caught_up(&source_watermark, &target_watermark)?;

    let mut tables = Vec::new();
    let selected_tables =
        selected_configured_tables(&config.dataset.tables, table_filter, "verify --table")?;

    for table in selected_tables {
        let relation = table.relation_id();
        let verify = table.verify.as_ref().cloned().unwrap_or_default();
        let row_filter = verify.row_filter.clone();
        let source_snapshot = snapshot_postgres_table(PostgresSnapshotConfig {
            database_url: config.source.database_url.expose().to_string(),
            relation: relation.clone(),
            primary_key: verify.primary_key.clone(),
            excluded_columns: verify.excluded_columns.clone(),
            row_filter: row_filter.clone(),
            watermark_lsn: source_watermark.clone(),
        })
        .await?;
        let target_snapshot = snapshot_postgres_table(PostgresSnapshotConfig {
            database_url: target_database_url.to_string(),
            relation,
            primary_key: verify.primary_key,
            excluded_columns: verify.excluded_columns,
            row_filter: row_filter.clone(),
            watermark_lsn: target_watermark.clone(),
        })
        .await?;
        tables.push(TableVerifySummary::from_comparison(
            compare_snapshots(source_snapshot, target_snapshot),
            row_filter,
        ));
    }

    let summary = VerifySummary {
        source_watermark_lsn: source_watermark,
        target_watermark_lsn: target_watermark,
        converged: tables.iter().all(|table| table.converged),
        checksum_status: ChecksumStatus::from_validation(Some(
            tables
                .iter()
                .all(|table| table.checksum_status == ChecksumStatus::Match),
        )),
        tables,
    };
    target_store
        .record_validation_event(ValidationEvent {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            source_watermark_lsn: summary.source_watermark_lsn.clone(),
            target_watermark_lsn: summary.target_watermark_lsn.clone(),
            converged: summary.converged,
            table_count: usize_to_i64_count("validation table_count", summary.tables.len())?,
            drift_count: usize_to_i64_count(
                "validation drift_count",
                summary
                    .tables
                    .iter()
                    .filter(|table| !table.converged)
                    .count(),
            )?,
            drift_relations: summary
                .tables
                .iter()
                .filter(|table| !table.converged)
                .map(|table| table.relation.clone())
                .collect(),
            evidence_sha256: Some(validation_evidence_sha256(&summary.tables)),
            completed_at: String::new(),
        })
        .await?;
    if table_filter.is_none() && summary.converged {
        let flow = FlowKey::new(&config.source.id, &config.dataset.id);
        if let Some(run) = target_store.load_latest_snapshot_run(&flow).await? {
            let handoff = target_store
                .load_latest_snapshot_handoff_event(&flow)
                .await?;
            if let Some(verified) = verified_snapshot_run(config, &run, handoff.as_ref()) {
                target_store.transition_snapshot_run(verified).await?;
            }
        }
    }

    Ok(summary)
}
