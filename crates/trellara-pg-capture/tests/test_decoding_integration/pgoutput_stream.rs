use super::support::*;
use trellara_pg_capture::{
    ChangeSource, PgCapture, PgCaptureConfig, PgOutputDecoder, PgOutputStreamCapture,
    TableSelector, TransactionAssembler, TransactionAssemblerConfig,
};
use trellara_protocol::{Operation, TransactionEnvelope};

#[tokio::test]
async fn pgoutput_sql_binary_changes_decode_to_transaction_envelope() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_publication(&database_url, PGOUTPUT_SQL_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_SQL_SLOT_NAME).await?;
    PgCapture::bootstrap(pgoutput_config(
        &database_url,
        PGOUTPUT_SQL_PUBLICATION_NAME,
        PGOUTPUT_SQL_SLOT_NAME,
    ))
    .await?;

    let client = connect(&database_url).await?;
    client
        .execute(
            r#"
            insert into public.sales (id, store_id, customer_id, amount_cents, status)
            values ($1, $2, $3, $4, $5)
            on conflict (id) do update
              set amount_cents = excluded.amount_cents,
                  status = excluded.status
            "#,
            &[
                &"pgoutput-sql-sale",
                &"store-pgoutput",
                &"customer-pgoutput",
                &"4200",
                &"paid",
            ],
        )
        .await?;

    let rows = client
        .query(
            r#"
            select data
              from pg_logical_slot_get_binary_changes(
                   $1, null, null,
                   'proto_version', '1',
                   'publication_names', $2
              )
            "#,
            &[&PGOUTPUT_SQL_SLOT_NAME, &PGOUTPUT_SQL_PUBLICATION_NAME],
        )
        .await?;

    let mut decoder = PgOutputDecoder::default();
    let mut assembler = TransactionAssembler::default();
    let assembler_config = TransactionAssemblerConfig {
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
    };
    let mut envelope = None;
    for row in rows {
        let data = row.get::<_, Vec<u8>>("data");
        if let Some(event) = decoder.decode(&data)? {
            if let Some(committed) = assembler.apply(&assembler_config, event)? {
                envelope = Some(committed);
            }
        }
    }

    assert_pgoutput_envelope(envelope.expect("committed pgoutput transaction"))?;

    client
        .execute(
            "delete from public.sales where id = $1",
            &[&"pgoutput-sql-sale"],
        )
        .await?;
    reset_publication(&database_url, PGOUTPUT_SQL_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_SQL_SLOT_NAME).await?;
    Ok(())
}

#[tokio::test]
async fn pgoutput_stream_capture_emits_committed_transaction() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_publication(&database_url, PGOUTPUT_STREAM_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_STREAM_SLOT_NAME).await?;

    let mut capture = PgOutputStreamCapture::connect(pgoutput_config(
        &database_url,
        PGOUTPUT_STREAM_PUBLICATION_NAME,
        PGOUTPUT_STREAM_SLOT_NAME,
    ))
    .await?;

    let client = connect(&database_url).await?;
    client
        .execute(
            r#"
            insert into public.sales (id, store_id, customer_id, amount_cents, status)
            values ($1, $2, $3, $4, $5)
            on conflict (id) do update
              set amount_cents = excluded.amount_cents,
                  status = excluded.status
            "#,
            &[
                &"pgoutput-stream-sale",
                &"store-pgoutput",
                &"customer-pgoutput",
                &"5100",
                &"paid",
            ],
        )
        .await?;

    let envelope = capture
        .next_transaction()
        .await?
        .expect("captured pgoutput stream transaction");
    assert_pgoutput_envelope(envelope)?;

    client
        .execute(
            "delete from public.sales where id = $1",
            &[&"pgoutput-stream-sale"],
        )
        .await?;
    drop(capture);
    reset_publication(&database_url, PGOUTPUT_STREAM_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_STREAM_SLOT_NAME).await?;
    Ok(())
}

#[tokio::test]
async fn pgoutput_stream_capture_returns_none_when_idle() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_publication(&database_url, PGOUTPUT_STREAM_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_STREAM_SLOT_NAME).await?;

    let mut capture = PgOutputStreamCapture::connect(pgoutput_config(
        &database_url,
        PGOUTPUT_STREAM_PUBLICATION_NAME,
        PGOUTPUT_STREAM_SLOT_NAME,
    ))
    .await?;

    assert!(capture.next_transaction().await?.is_none());

    drop(capture);
    reset_publication(&database_url, PGOUTPUT_STREAM_PUBLICATION_NAME).await?;
    reset_slot(&database_url, PGOUTPUT_STREAM_SLOT_NAME).await?;
    Ok(())
}

fn pgoutput_config(database_url: &str, publication_name: &str, slot_name: &str) -> PgCaptureConfig {
    PgCaptureConfig {
        connection_uri: database_url.to_string(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: publication_name.to_string(),
        slot_name: slot_name.to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: true,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    }
}

fn assert_pgoutput_envelope(envelope: TransactionEnvelope) -> TestResult<()> {
    assert_eq!(envelope.source_id, "source-integration");
    assert_eq!(envelope.dataset_id, "retail-sales");
    assert!(envelope.commit_lsn.contains('/'));
    assert!(envelope.changes.iter().any(|change| matches!(
        Operation::try_from(change.operation),
        Ok(Operation::Insert | Operation::Update)
    )));
    envelope.verify_checksum()?;
    Ok(())
}
