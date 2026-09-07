use super::{
    async_trait, sleep, Arc, Duration, FlowKey, Mutex, NoTls, PostgresCheckpointStore, PublishAck,
    StreamMessage, StreamPublisher, TestResult, DATASET_ID, SLOT_NAME, SOURCE_DATABASE_URL_ENV,
    SOURCE_ID,
};
use trellara_checkpoint::CheckpointStore;

pub(crate) fn integration_database_url() -> Option<String> {
    std::env::var(SOURCE_DATABASE_URL_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

pub(crate) async fn connect(database_url: &str) -> TestResult<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("relay integration postgres connection failed: {error}");
        }
    });
    Ok(client)
}

pub(crate) async fn reset_source_state(database_url: &str) -> TestResult<()> {
    reset_source_row(database_url, "relay-capture-sale-1").await?;
    reset_slot(database_url, SLOT_NAME).await?;
    Ok(())
}

pub(crate) async fn reset_source_row(database_url: &str, id: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute("delete from public.sales where id = $1", &[&id])
        .await?;
    Ok(())
}

pub(crate) async fn reset_slot(database_url: &str, slot_name: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            r#"
            select pg_drop_replication_slot($1)
             where exists (
                 select 1
                   from pg_replication_slots
                  where slot_name = $1
             )
            "#,
            &[&slot_name],
        )
        .await?;
    Ok(())
}

pub(crate) async fn reset_publication(
    database_url: &str,
    publication_name: &str,
) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(&format!(
            "drop publication if exists {}",
            quote_test_ident(publication_name)
        ))
        .await?;
    Ok(())
}

pub(crate) async fn reset_checkpoint(
    database_url: &str,
    store: &PostgresCheckpointStore,
) -> TestResult<()> {
    let client = connect(database_url).await?;
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
    assert!(store
        .load_checkpoint(&FlowKey::new(SOURCE_ID, DATASET_ID))
        .await?
        .is_none());
    Ok(())
}

pub(crate) async fn write_source_transaction(database_url: &str) -> TestResult<()> {
    write_source_transaction_with_id(database_url, "relay-capture-sale-1").await
}

pub(crate) async fn write_source_transaction_with_id(
    database_url: &str,
    id: &str,
) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            r#"
            insert into public.sales (id, store_id, customer_id, amount_cents, status)
            values ($1, $2, $3, $4, $5)
            "#,
            &[&id, &"store-1", &"customer-1", &"3300", &"paid"],
        )
        .await?;
    Ok(())
}

pub(crate) async fn wait_for_confirmed_flush_lsn(
    database_url: &str,
    slot_name: &str,
    durable_lsn: &str,
) -> TestResult<()> {
    let client = connect(database_url).await?;
    for _ in 0..20 {
        let confirmed = client
            .query_one(
                r#"
                select coalesce(pg_wal_lsn_diff(confirmed_flush_lsn, $2::pg_lsn) >= 0, false)
                  from pg_replication_slots
                 where slot_name = $1
                "#,
                &[&slot_name, &durable_lsn],
            )
            .await?
            .get::<_, bool>(0);
        if confirmed {
            return Ok(());
        }
        sleep(Duration::from_millis(50)).await;
    }

    Err(format!("slot {slot_name} did not confirm durable LSN {durable_lsn}").into())
}

fn quote_test_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

#[derive(Clone)]
pub(crate) struct RecordingPublisher {
    published: Arc<Mutex<Vec<StreamMessage>>>,
}

impl RecordingPublisher {
    pub(crate) fn succeeding() -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(crate) fn published_messages(&self) -> Vec<StreamMessage> {
        self.published.lock().expect("publisher lock").clone()
    }
}

#[async_trait]
impl StreamPublisher for RecordingPublisher {
    async fn publish(&self, message: StreamMessage) -> trellara_stream::Result<PublishAck> {
        let mut published = self.published.lock().expect("publisher lock");
        let offset = published.len() as i64;
        let topic = message.topic.clone();
        let partition = message.partition.unwrap_or_default();
        published.push(message);
        Ok(PublishAck {
            topic,
            partition,
            offset,
        })
    }
}
