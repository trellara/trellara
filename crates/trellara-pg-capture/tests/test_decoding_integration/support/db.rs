use tokio_postgres::NoTls;

use super::{
    TestResult, DEFAULT_DATABASE_URL, SLOT_NAME, TEST_DATABASE_URL_ENV, TRUNCATE_TABLE_NAME,
};

pub(crate) fn integration_database_url() -> Option<String> {
    std::env::var(TEST_DATABASE_URL_ENV)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

pub(crate) async fn connect(database_url: &str) -> TestResult<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("test source postgres connection failed: {error}");
        }
    });
    Ok(client)
}

pub(crate) async fn reset_slot_and_row(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            "delete from public.sales where id = $1",
            &[&"capture-sale-1"],
        )
        .await?;
    reset_slot(database_url, SLOT_NAME).await?;
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

fn quote_test_ident(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

pub(crate) async fn reset_preflight_table(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(
            r#"
            drop table if exists public.trellara_preflight_no_pk;
            drop table if exists public.trellara_preflight_default_pk;
            "#,
        )
        .await?;
    Ok(())
}

pub(crate) async fn create_unsafe_preflight_table(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(
            r#"
            create table public.trellara_preflight_default_pk (
                id text primary key,
                value text not null
            );

            create table public.trellara_preflight_no_pk (
                id text not null,
                value text not null
            )
            "#,
        )
        .await?;
    Ok(())
}

pub(crate) async fn write_source_transaction(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            r#"
            insert into public.sales (id, store_id, customer_id, amount_cents, status)
            values ($1, $2, $3, $4, $5)
            "#,
            &[
                &"capture-sale-1",
                &"store-1",
                &"customer-1",
                &"2500",
                &"paid",
            ],
        )
        .await?;
    Ok(())
}

pub(crate) async fn reset_truncate_table(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(&format!(
            r#"
            drop table if exists public.{TRUNCATE_TABLE_NAME};
            create table public.{TRUNCATE_TABLE_NAME} (
                id text primary key,
                value text not null
            );
            alter table public.{TRUNCATE_TABLE_NAME} replica identity full;
            insert into public.{TRUNCATE_TABLE_NAME} (id, value)
            values ('truncate-1', 'before');
            "#
        ))
        .await?;
    Ok(())
}

pub(crate) async fn write_truncate_transaction(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(&format!("truncate table public.{TRUNCATE_TABLE_NAME}"))
        .await?;
    Ok(())
}

#[test]
fn default_source_database_url_is_documented() {
    assert_eq!(
        DEFAULT_DATABASE_URL,
        "postgresql://trellara:trellara@localhost:55432/trellara_source"
    );
}
