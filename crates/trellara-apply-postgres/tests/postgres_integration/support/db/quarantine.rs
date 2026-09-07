use super::*;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct QuarantineRow {
    pub(crate) transaction_id: String,
    pub(crate) commit_lsn: String,
    pub(crate) reason: String,
    pub(crate) detail: String,
    pub(crate) attempt_count: i64,
}

pub(crate) async fn latest_quarantine(
    client: &tokio_postgres::Client,
) -> TestResult<Option<QuarantineRow>> {
    Ok(client
        .query_opt(
            r#"
            select transaction_id,
                   commit_lsn,
                   reason,
                   detail,
                   attempt_count
              from trellara.apply_quarantine
             where source_id = $1
               and dataset_id = $2
             order by last_seen_at desc
             limit 1
            "#,
            &[&SOURCE_ID, &DATASET_ID],
        )
        .await?
        .map(|row| QuarantineRow {
            transaction_id: row.get(0),
            commit_lsn: row.get(1),
            reason: row.get(2),
            detail: row.get(3),
            attempt_count: row.get(4),
        }))
}
