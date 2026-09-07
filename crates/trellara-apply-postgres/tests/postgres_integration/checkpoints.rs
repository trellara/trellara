use super::support::*;
use trellara_apply_postgres::ApplyDecision;

#[tokio::test]
async fn applies_transaction_once_and_records_checkpoint() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    let envelope = envelope(
        "tx-integration-1",
        "0/16B6C50",
        "0/16B6D28",
        vec![insert_change("tx-integration-1", 1, "sale-1", "1299")],
    );

    let first = applier.apply_envelope(&envelope).await?;
    assert_eq!(first.decision, ApplyDecision::Applied);
    assert_eq!(first.applied_changes, 1);
    assert_eq!(
        sale_amount(&client, "sale-1").await?,
        Some("1299".to_string())
    );

    let second = applier.apply_envelope(&envelope).await?;
    assert_eq!(second.decision, ApplyDecision::SkippedDuplicate);
    assert_eq!(second.applied_changes, 0);
    assert_eq!(sale_count(&client, "sale-1").await?, 1);
    assert_eq!(
        applied_checkpoint_lsn(&client).await?,
        Some("0/16B6D28".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn replaying_older_transaction_does_not_move_checkpoint_backwards() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;

    let newer = envelope(
        "tx-monotonic-newer",
        "0/16B8000",
        "0/16B9000",
        vec![insert_change("tx-monotonic-newer", 1, "sale-newer", "9000")],
    );
    let older_replay = envelope(
        "tx-monotonic-older",
        "0/16B7000",
        "0/16B8000",
        vec![insert_change("tx-monotonic-older", 1, "sale-older", "8000")],
    );

    assert_eq!(
        applier.apply_envelope(&newer).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        applied_checkpoint_lsn(&client).await?,
        Some("0/16B9000".to_string())
    );

    assert_eq!(
        applier.apply_envelope(&older_replay).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        applied_checkpoint_lsn(&client).await?,
        Some("0/16B9000".to_string())
    );
    assert_eq!(
        sale_amount(&client, "sale-older").await?,
        Some("8000".to_string())
    );

    Ok(())
}
