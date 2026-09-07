use super::support::*;
use trellara_apply_postgres::ApplyDecision;

#[tokio::test]
async fn manifest_envelope_records_partition_checkpoints_atomically() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    let envelope = with_partition_manifest(
        envelope(
            "tx-partition-checkpoints",
            "0/16B7200",
            "0/16B7600",
            vec![
                insert_change("tx-partition-checkpoints", 1, "sale-partition-1", "100"),
                insert_change("tx-partition-checkpoints", 2, "sale-partition-2", "200"),
            ],
        ),
        vec![manifest_partition(1, 1, 1), manifest_partition(3, 2, 2)],
    );

    let outcome = applier.apply_envelope(&envelope).await?;

    assert_eq!(outcome.decision, ApplyDecision::Applied);
    assert_eq!(
        partition_checkpoint_lsn(&client, 1).await?,
        Some("0/16B7600".to_string())
    );
    assert_eq!(
        partition_checkpoint_lsn(&client, 3).await?,
        Some("0/16B7600".to_string())
    );
    assert_eq!(partition_checkpoint_lsn(&client, 0).await?, None);

    Ok(())
}

#[tokio::test]
async fn replaying_older_manifest_does_not_move_partition_checkpoint_backwards() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    let newer = with_partition_manifest(
        envelope(
            "tx-partition-newer",
            "0/16B8000",
            "0/16B9000",
            vec![insert_change(
                "tx-partition-newer",
                1,
                "sale-partition-newer",
                "9000",
            )],
        ),
        vec![manifest_partition(1, 1, 1)],
    );
    let older_replay = with_partition_manifest(
        envelope(
            "tx-partition-older",
            "0/16B7000",
            "0/16B7800",
            vec![insert_change(
                "tx-partition-older",
                1,
                "sale-partition-older",
                "7800",
            )],
        ),
        vec![manifest_partition(1, 1, 1)],
    );

    assert_eq!(
        applier.apply_envelope(&newer).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        partition_checkpoint_lsn(&client, 1).await?,
        Some("0/16B9000".to_string())
    );
    assert_eq!(
        applier.apply_envelope(&older_replay).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        partition_checkpoint_lsn(&client, 1).await?,
        Some("0/16B9000".to_string())
    );
    assert_eq!(
        sale_amount(&client, "sale-partition-older").await?,
        Some("7800".to_string())
    );

    Ok(())
}
