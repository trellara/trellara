use super::*;

type TestResult<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

const SOURCE_DATABASE_URL_ENV: &str = "TRELLARA_SOURCE_TEST_DATABASE_URL";
const TARGET_DATABASE_URL_ENV: &str = "TRELLARA_TARGET_TEST_DATABASE_URL";
const LIFETIME_TABLE: &str = "trellara_cli_snapshot_lifetime_sales";
const LIFETIME_PUBLICATION: &str = "trellara_cli_snapshot_lifetime_publication";
const LIFETIME_SLOT: &str = "trellara_cli_snapshot_lifetime_slot";

#[test]
fn snapshot_exported_run_record_preserves_exported_snapshot_boundary() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let bootstrap = verified_bootstrap(Some("00000003-0000001B-1"));

    let run = snapshot_exported_run_record(&config, "snapshot-run-1", &bootstrap, "0/16B8000", 12)
        .expect("snapshot exported run");

    assert_eq!(run.state, SnapshotRunState::SnapshotExported);
    assert_eq!(run.slot_name, "trellara_slot");
    assert_eq!(run.consistent_lsn.as_deref(), Some("0/16B8000"));
    assert_eq!(run.current_relation, None);
    assert_eq!(run.copied_rows, 12);
    assert_eq!(run.failure_reason, None);
}

#[test]
fn snapshot_exported_run_record_is_absent_without_exported_snapshot() {
    let config = TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse");
    let bootstrap = verified_bootstrap(None);

    assert!(
        snapshot_exported_run_record(&config, "snapshot-run-1", &bootstrap, "0/16B8000", 12)
            .is_none()
    );
}

#[tokio::test]
async fn snapshot_copy_keeps_exported_slot_holder_until_reseed_imports_snapshot() -> TestResult<()>
{
    let Some(source_url) = integration_database_url(SOURCE_DATABASE_URL_ENV) else {
        return Ok(());
    };
    let Some(target_url) = integration_database_url(TARGET_DATABASE_URL_ENV) else {
        return Ok(());
    };
    reset_lifetime_table(&source_url).await?;
    reset_lifetime_table(&target_url).await?;
    reset_publication(&source_url, LIFETIME_PUBLICATION).await?;
    reset_slot(&source_url, LIFETIME_SLOT).await?;
    insert_lifetime_row(&source_url, "snapshot-lifetime-visible", "1299").await?;

    let run_id = format!("snapshot-lifetime-{}", unique_suffix());
    let config = TrellaraConfig::from_yaml(
        &snapshot_lifetime_yaml(&source_url, &target_url),
        "snapshot-lifetime-test",
    )?;

    let summary = snapshot_configured_tables(
        &config,
        &target_url,
        Some(&run_id),
        Some(&format!("public.{LIFETIME_TABLE}")),
        true,
        false,
    )
    .await?;

    assert_eq!(
        summary.state,
        SnapshotRunState::StreamHandoffReady.to_string()
    );
    assert_eq!(summary.copied_rows, 1);
    assert!(summary
        .consistency_note
        .contains("exported logical snapshot"));
    assert_eq!(
        lifetime_rows(&target_url).await?,
        vec![("snapshot-lifetime-visible".to_string(), "1299".to_string())]
    );

    reset_publication(&source_url, LIFETIME_PUBLICATION).await?;
    reset_slot(&source_url, LIFETIME_SLOT).await?;
    reset_lifetime_table(&source_url).await?;
    reset_lifetime_table(&target_url).await?;
    Ok(())
}

fn integration_database_url(env: &str) -> Option<String> {
    std::env::var(env)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn snapshot_lifetime_yaml(source_url: &str, target_url: &str) -> String {
    format!(
        r#"
source:
  id: cli-snapshot-lifetime-source
  database_url: {}
  publication: {LIFETIME_PUBLICATION}
  slot: {LIFETIME_SLOT}
  pgoutput:
    protocol_version: 2
    streaming: true
dataset:
  id: cli-snapshot-lifetime-dataset
  mode: strict_transaction_order
  tables:
    - schema: public
      name: {LIFETIME_TABLE}
      verify:
        primary_key: id
        row_filter: "id like 'snapshot-lifetime-%'"
stream:
  kind: kafka
  bootstrap_servers: localhost:9092
  topic: trellara.cli-snapshot-lifetime.strict
target:
  database_url: {}
"#,
        yaml_quote(source_url),
        yaml_quote(target_url)
    )
}

fn yaml_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

async fn connect(database_url: &str) -> TestResult<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(database_url, tokio_postgres::NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("cli snapshot integration postgres connection failed: {error}");
        }
    });
    Ok(client)
}

async fn reset_lifetime_table(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(&format!(
            r#"
            create table if not exists public.{LIFETIME_TABLE} (
                id text primary key,
                amount_cents text not null,
                updated_at text not null default 'snapshot-lifetime'
            );
            delete from public.{LIFETIME_TABLE}
             where id like 'snapshot-lifetime-%';
            "#
        ))
        .await?;
    Ok(())
}

async fn reset_publication(database_url: &str, publication_name: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(&format!(
            "drop publication if exists {}",
            quote_ident(publication_name)
        ))
        .await?;
    Ok(())
}

async fn reset_slot(database_url: &str, slot_name: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            "select pg_drop_replication_slot($1) where exists (select 1 from pg_replication_slots where slot_name = $1)",
            &[&slot_name],
        )
        .await?;
    Ok(())
}

async fn insert_lifetime_row(database_url: &str, id: &str, amount_cents: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            &format!("insert into public.{LIFETIME_TABLE} (id, amount_cents) values ($1, $2)"),
            &[&id, &amount_cents],
        )
        .await?;
    Ok(())
}

async fn lifetime_rows(database_url: &str) -> TestResult<Vec<(String, String)>> {
    let client = connect(database_url).await?;
    Ok(client
        .query(
            &format!(
                r#"
                select id, amount_cents
                  from public.{LIFETIME_TABLE}
                 where id like 'snapshot-lifetime-%'
                 order by id
                "#
            ),
            &[],
        )
        .await?
        .into_iter()
        .map(|row| (row.get(0), row.get(1)))
        .collect())
}

fn quote_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}
