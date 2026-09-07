use super::*;

pub(crate) async fn sale_amount(
    client: &tokio_postgres::Client,
    id: &str,
) -> TestResult<Option<String>> {
    Ok(client
        .query_opt(
            &format!("select amount_cents from public.{TABLE_NAME} where id = $1"),
            &[&id],
        )
        .await?
        .map(|row| row.get(0)))
}

pub(crate) async fn sale_reviewed_by(
    client: &tokio_postgres::Client,
    id: &str,
) -> TestResult<Option<String>> {
    Ok(client
        .query_opt(
            &format!("select reviewed_by from public.{TABLE_NAME} where id = $1"),
            &[&id],
        )
        .await?
        .map(|row| row.get(0)))
}

pub(crate) async fn sale_receipt_blob(
    client: &tokio_postgres::Client,
    id: &str,
) -> TestResult<Option<String>> {
    Ok(client
        .query_opt(
            &format!("select receipt_blob from public.{TABLE_NAME} where id = $1"),
            &[&id],
        )
        .await?
        .map(|row| row.get(0)))
}

pub(crate) async fn sale_receipt_bytes(
    client: &tokio_postgres::Client,
    id: &str,
) -> TestResult<Option<Vec<u8>>> {
    Ok(client
        .query_opt(
            &format!("select receipt_bytes from public.{TABLE_NAME} where id = $1"),
            &[&id],
        )
        .await?
        .map(|row| row.get(0)))
}

pub(crate) async fn sale_count(client: &tokio_postgres::Client, id: &str) -> TestResult<i64> {
    Ok(client
        .query_one(
            &format!("select count(*) from public.{TABLE_NAME} where id = $1"),
            &[&id],
        )
        .await?
        .get(0))
}

pub(crate) async fn total_sale_count(client: &tokio_postgres::Client) -> TestResult<i64> {
    Ok(client
        .query_one(&format!("select count(*) from public.{TABLE_NAME}"), &[])
        .await?
        .get(0))
}

pub(crate) async fn applied_checkpoint_lsn(
    client: &tokio_postgres::Client,
) -> TestResult<Option<String>> {
    Ok(client
        .query_opt(
            r#"
            select last_applied_lsn
              from trellara.flow_checkpoints
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?
        .map(|row| row.get(0)))
}

pub(crate) async fn applied_transaction_count(client: &tokio_postgres::Client) -> TestResult<i64> {
    Ok(client
        .query_one(
            r#"
            select count(*)
              from trellara.applied_transactions
             where source_id = $1
               and dataset_id = $2
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?
        .get(0))
}

pub(crate) async fn partition_checkpoint_lsn(
    client: &tokio_postgres::Client,
    partition_id: i32,
) -> TestResult<Option<String>> {
    Ok(client
        .query_opt(
            r#"
            select last_applied_lsn
              from trellara.partition_checkpoints
             where source_id = $1
               and dataset_id = $2
               and partition_id = $3
            "#,
            &[&SOURCE_ID, &DATASET_ID, &partition_id],
        )
        .await?
        .map(|row| row.get(0)))
}
