use super::{TestResult, DATABASE_ID, DATASET_ID, SOURCE_ID, TEST_DATABASE_URL_ENV};

pub(crate) fn integration_database_url() -> Option<String> {
    match std::env::var(TEST_DATABASE_URL_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value),
        _ => None,
    }
}

pub(crate) async fn reset_store(database_url: &str) -> TestResult<()> {
    let (client, connection) = tokio_postgres::connect(database_url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("test postgres checkpoint connection failed: {error}");
        }
    });
    client
        .execute(
            "delete from trellara.iceberg_commit_receipts where dataset_id = $1",
            &[&DATASET_ID],
        )
        .await?;
    client
        .execute(
            "delete from trellara.iceberg_commit_intents where dataset_id = $1",
            &[&DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.validation_events
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.snapshot_table_progress
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.snapshot_runs
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.snapshot_handoff_events
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.reseed_events
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.apply_quarantine
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.applied_transactions
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.partition_checkpoints
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    client
        .execute(
            r#"
            delete from trellara.flow_checkpoints
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?;
    Ok(())
}

pub(crate) async fn insert_quarantine(
    database_url: &str,
    transaction_id: &str,
    commit_lsn: &str,
) -> TestResult<()> {
    let (client, connection) = tokio_postgres::connect(database_url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("test postgres checkpoint connection failed: {error}");
        }
    });
    client
        .execute(
            r#"
            insert into trellara.apply_quarantine
                (source_id, database_id, dataset_id, transaction_id, commit_lsn, reason, detail)
            values ($1, $2, $3, $4, $5, $6, $7)
            "#,
            &[
                &SOURCE_ID,
                &DATABASE_ID,
                &DATASET_ID,
                &transaction_id,
                &commit_lsn,
                &"target_postgres_error",
                &"target table is missing",
            ],
        )
        .await?;
    Ok(())
}
