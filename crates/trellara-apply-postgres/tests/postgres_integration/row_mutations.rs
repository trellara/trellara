use super::support::*;
use trellara_apply_postgres::ApplyDecision;

#[tokio::test]
async fn applies_insert_update_delete_as_distinct_transactions() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;

    let insert = envelope(
        "tx-integration-2",
        "0/16B6D28",
        "0/16B6E00",
        vec![insert_change("tx-integration-2", 1, "sale-2", "500")],
    );
    let update = envelope(
        "tx-integration-3",
        "0/16B6E00",
        "0/16B6F80",
        vec![update_change("tx-integration-3", 1, "sale-2", "500", "750")],
    );
    let delete = envelope(
        "tx-integration-4",
        "0/16B6F80",
        "0/16B7000",
        vec![delete_change("tx-integration-4", 1, "sale-2", "750")],
    );

    assert_eq!(
        applier.apply_envelope(&insert).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        sale_amount(&client, "sale-2").await?,
        Some("500".to_string())
    );

    assert_eq!(
        applier.apply_envelope(&update).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        sale_amount(&client, "sale-2").await?,
        Some("750".to_string())
    );

    assert_eq!(
        applier.apply_envelope(&delete).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(sale_amount(&client, "sale-2").await?, None);
    assert_eq!(
        applied_checkpoint_lsn(&client).await?,
        Some("0/16B7000".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn key_changing_update_moves_primary_key_with_old_key_predicate() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    client
        .execute(
            &format!("insert into public.{TABLE_NAME} (id, amount_cents) values ($1, $2)"),
            &[&"sale-key-old", &"100"],
        )
        .await?;
    let update = envelope(
        "tx-key-change-1",
        "0/16B7040",
        "0/16B7050",
        vec![key_changing_update(
            "tx-key-change-1",
            1,
            "sale-key-old",
            "sale-key-new",
            "150",
        )],
    );

    let outcome = applier.apply_envelope(&update).await?;

    assert_eq!(outcome.decision, ApplyDecision::Applied);
    assert_eq!(sale_amount(&client, "sale-key-old").await?, None);
    assert_eq!(
        sale_amount(&client, "sale-key-new").await?,
        Some("150".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn applies_truncate_transaction_and_records_checkpoint() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;

    let seed = envelope(
        "tx-integration-truncate-seed",
        "0/16B7000",
        "0/16B7100",
        vec![
            insert_change("tx-integration-truncate-seed", 1, "sale-truncate-1", "100"),
            insert_change("tx-integration-truncate-seed", 2, "sale-truncate-2", "200"),
        ],
    );
    let truncate = envelope(
        "tx-integration-truncate",
        "0/16B7100",
        "0/16B7200",
        vec![truncate_change("tx-integration-truncate", 1)],
    );

    assert_eq!(
        applier.apply_envelope(&seed).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(total_sale_count(&client).await?, 2);

    let outcome = applier.apply_envelope(&truncate).await?;

    assert_eq!(outcome.decision, ApplyDecision::Applied);
    assert_eq!(outcome.applied_changes, 1);
    assert_eq!(total_sale_count(&client).await?, 0);
    assert_eq!(
        applied_checkpoint_lsn(&client).await?,
        Some("0/16B7200".to_string())
    );

    Ok(())
}
