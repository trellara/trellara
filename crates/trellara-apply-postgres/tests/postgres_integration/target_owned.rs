use super::support::*;
use trellara_apply_postgres::ApplyDecision;

#[tokio::test]
async fn preserves_target_owned_columns_during_insert_and_update() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_policy_applier(&database_url).await?;
    reset_target_table(&client).await?;

    let insert = envelope(
        "tx-target-owned-1",
        "0/16B7010",
        "0/16B7020",
        vec![insert_change_with_review(
            "tx-target-owned-1",
            1,
            "sale-owned-1",
            "900",
            "source-user",
        )],
    );
    assert_eq!(
        applier.apply_envelope(&insert).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        sale_amount(&client, "sale-owned-1").await?,
        Some("900".to_string())
    );
    assert_eq!(
        sale_reviewed_by(&client, "sale-owned-1").await?,
        Some("target-default".to_string())
    );

    client
        .execute(
            &format!("update public.{TABLE_NAME} set reviewed_by = $1 where id = $2"),
            &[&"manager", &"sale-owned-1"],
        )
        .await?;
    let update = envelope(
        "tx-target-owned-2",
        "0/16B7020",
        "0/16B7030",
        vec![update_change_with_review(
            "tx-target-owned-2",
            1,
            "sale-owned-1",
            "900",
            "950",
            "source-update",
        )],
    );

    assert_eq!(
        applier.apply_envelope(&update).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        sale_amount(&client, "sale-owned-1").await?,
        Some("950".to_string())
    );
    assert_eq!(
        sale_reviewed_by(&client, "sale-owned-1").await?,
        Some("manager".to_string())
    );

    Ok(())
}

#[tokio::test]
async fn update_preserves_omitted_unchanged_toast_columns() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    let receipt_blob = "source receipt payload ".repeat(1_000);
    client
        .execute(
            &format!(
                "insert into public.{TABLE_NAME} (id, amount_cents, receipt_blob) values ($1, $2, $3)"
            ),
            &[&"sale-toast-1", &"100", &receipt_blob],
        )
        .await?;
    let update = envelope(
        "tx-toast-omitted-1",
        "0/16B7030",
        "0/16B7040",
        vec![update_change(
            "tx-toast-omitted-1",
            1,
            "sale-toast-1",
            "100",
            "150",
        )],
    );

    let outcome = applier.apply_envelope(&update).await?;

    assert_eq!(outcome.decision, ApplyDecision::Applied);
    assert_eq!(
        sale_amount(&client, "sale-toast-1").await?,
        Some("150".to_string())
    );
    assert_eq!(
        sale_receipt_blob(&client, "sale-toast-1").await?,
        Some(receipt_blob)
    );

    Ok(())
}

#[tokio::test]
async fn applies_binary_bytea_values() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    let insert = envelope(
        "tx-bytea-insert",
        "0/16B7040",
        "0/16B7050",
        vec![insert_change_with_receipt_bytes(
            "tx-bytea-insert",
            1,
            "sale-bytea-1",
            "900",
            &[1, 2, 3, 255],
        )],
    );
    let update = envelope(
        "tx-bytea-update",
        "0/16B7050",
        "0/16B7060",
        vec![update_change_with_receipt_bytes(
            "tx-bytea-update",
            1,
            "sale-bytea-1",
            "900",
            "950",
            &[4, 5, 6, 0],
        )],
    );

    assert_eq!(
        applier.apply_envelope(&insert).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        sale_receipt_bytes(&client, "sale-bytea-1").await?,
        Some(vec![1, 2, 3, 255])
    );

    assert_eq!(
        applier.apply_envelope(&update).await?.decision,
        ApplyDecision::Applied
    );
    assert_eq!(
        sale_amount(&client, "sale-bytea-1").await?,
        Some("950".to_string())
    );
    assert_eq!(
        sale_receipt_bytes(&client, "sale-bytea-1").await?,
        Some(vec![4, 5, 6, 0])
    );

    Ok(())
}
