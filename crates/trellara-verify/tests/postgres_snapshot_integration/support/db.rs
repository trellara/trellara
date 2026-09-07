use tokio_postgres::NoTls;
use trellara_verify::RowSnapshot;

use super::{TestResult, RESEED_TABLE};

pub(crate) fn integration_database_url(env: &str) -> Option<String> {
    std::env::var(env)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

pub(crate) async fn connect(database_url: &str) -> TestResult<tokio_postgres::Client> {
    let (client, connection) = tokio_postgres::connect(database_url, NoTls).await?;
    tokio::spawn(async move {
        if let Err(error) = connection.await {
            eprintln!("verify integration postgres connection failed: {error}");
        }
    });
    Ok(client)
}

pub(crate) async fn reset_rows(database_url: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            "delete from public.sales where id in ($1, $2)",
            &[&"verify-sale-1", &"verify-sale-2"],
        )
        .await?;
    Ok(())
}

pub(crate) async fn reset_reseed_table(database_url: &str, source: bool) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .batch_execute(&format!(
            r#"
            create table if not exists public.{RESEED_TABLE} (
                id text primary key,
                store_id text not null,
                amount_cents text not null,
                reviewed_by text not null default 'target-default',
                receipt_bytes bytea not null default '\x'::bytea,
                updated_at timestamptz not null default now()
            );
            alter table public.{RESEED_TABLE}
                add column if not exists receipt_bytes bytea not null default '\x'::bytea;
            truncate table public.{RESEED_TABLE};
            "#
        ))
        .await?;
    if source {
        client
            .execute(
                &format!(
                    "alter table public.{RESEED_TABLE} alter column reviewed_by set default 'source-default'"
                ),
                &[],
            )
            .await?;
    }
    Ok(())
}

pub(crate) async fn reset_slot(database_url: &str, slot_name: &str) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute("select pg_drop_replication_slot($1) where exists (select 1 from pg_replication_slots where slot_name = $1)", &[&slot_name])
        .await?;
    Ok(())
}

pub(crate) async fn insert_row(
    database_url: &str,
    id: &str,
    store_id: &str,
    amount_cents: &str,
) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            r#"
            insert into public.sales (id, store_id, customer_id, amount_cents, status)
            values ($1, $2, $3, $4, $5)
            "#,
            &[&id, &store_id, &"customer-verify", &amount_cents, &"paid"],
        )
        .await?;
    Ok(())
}

pub(crate) async fn insert_reseed_row(
    database_url: &str,
    id: &str,
    store_id: &str,
    amount_cents: &str,
    reviewed_by: &str,
) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            &format!(
                r#"
                insert into public.{RESEED_TABLE} (id, store_id, amount_cents, reviewed_by)
                values ($1, $2, $3, $4)
                "#
            ),
            &[&id, &store_id, &amount_cents, &reviewed_by],
        )
        .await?;
    Ok(())
}

pub(crate) async fn insert_reseed_row_with_bytes(
    database_url: &str,
    id: &str,
    store_id: &str,
    amount_cents: &str,
    reviewed_by: &str,
    receipt_bytes: &[u8],
) -> TestResult<()> {
    let client = connect(database_url).await?;
    client
        .execute(
            &format!(
                r#"
                insert into public.{RESEED_TABLE} (id, store_id, amount_cents, reviewed_by, receipt_bytes)
                values ($1, $2, $3, $4, $5)
                "#
            ),
            &[&id, &store_id, &amount_cents, &reviewed_by, &receipt_bytes],
        )
        .await?;
    Ok(())
}

pub(crate) async fn reseed_rows(
    database_url: &str,
) -> TestResult<Vec<(String, String, String, String)>> {
    let client = connect(database_url).await?;
    Ok(client
        .query(
            &format!(
                r#"
                select id, store_id, amount_cents, reviewed_by
                  from public.{RESEED_TABLE}
                 order by id
                "#
            ),
            &[],
        )
        .await?
        .into_iter()
        .map(|row| (row.get(0), row.get(1), row.get(2), row.get(3)))
        .collect())
}

pub(crate) async fn reseed_row_bytes(database_url: &str, id: &str) -> TestResult<Option<Vec<u8>>> {
    let client = connect(database_url).await?;
    Ok(client
        .query_opt(
            &format!("select receipt_bytes from public.{RESEED_TABLE} where id = $1"),
            &[&id],
        )
        .await?
        .map(|row| row.get(0)))
}

pub(crate) fn expected_row(id: &str, store_id: &str, amount_cents: &str) -> RowSnapshot {
    RowSnapshot::from_json_value(
        id,
        &serde_json::json!({
            "id": id,
            "store_id": store_id,
            "customer_id": "customer-verify",
            "amount_cents": amount_cents,
            "status": "paid"
        }),
    )
    .expect("expected row")
}
