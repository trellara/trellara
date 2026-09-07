use super::support::*;
use trellara_apply_postgres::{ApplyDecision, ApplyError};
use trellara_protocol::RelationId;

#[tokio::test]
async fn missing_target_table_fails_closed_without_checkpoint_or_dedup() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    client
        .batch_execute("drop table if exists public.trellara_apply_missing_sales")
        .await?;
    let envelope = envelope(
        "tx-missing-target",
        "0/16B7100",
        "0/16B7200",
        vec![insert_change_for_relation(
            "tx-missing-target",
            1,
            RelationId::new(99, "public", "trellara_apply_missing_sales"),
            "missing-sale-1",
            "999",
        )],
    );

    let result = applier.apply_envelope(&envelope).await;

    assert!(matches!(result, Err(ApplyError::Postgres(_))));
    assert_eq!(applied_checkpoint_lsn(&client).await?, None);
    assert_eq!(applied_transaction_count(&client).await?, 0);
    let quarantine = latest_quarantine(&client).await?.expect("quarantine row");
    assert_eq!(quarantine.transaction_id, "tx-missing-target");
    assert_eq!(quarantine.commit_lsn, "0/16B7200");
    assert_eq!(quarantine.reason, "target_postgres_error");
    assert!(quarantine.detail.contains("trellara_apply_missing_sales"));

    let retry = applier.apply_envelope(&envelope).await;
    assert!(matches!(retry, Err(ApplyError::Postgres(_))));
    assert_eq!(applied_checkpoint_lsn(&client).await?, None);
    assert_eq!(applied_transaction_count(&client).await?, 0);
    assert_eq!(
        latest_quarantine(&client)
            .await?
            .expect("quarantine row")
            .attempt_count,
        2
    );

    client
        .batch_execute("create table public.trellara_apply_missing_sales (id text primary key, amount_cents text not null)")
        .await?;
    let repaired = applier.apply_envelope(&envelope).await?;
    assert_eq!(repaired.decision, ApplyDecision::Applied);
    assert_eq!(repaired.applied_changes, 1);
    assert_eq!(latest_quarantine(&client).await?, None);
    assert_eq!(
        applied_checkpoint_lsn(&client).await?,
        Some("0/16B7200".to_string())
    );
    assert_eq!(applied_transaction_count(&client).await?, 1);
    Ok(())
}
