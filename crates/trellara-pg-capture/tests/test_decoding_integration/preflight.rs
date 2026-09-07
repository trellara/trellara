use super::support::*;
use trellara_pg_capture::{CaptureError, PgCapture, PgCaptureConfig, TableSelector};
use trellara_protocol::ReplicaIdentity;

#[tokio::test]
async fn capture_preflight_reports_replica_identity_safety() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    reset_preflight_table(&database_url).await?;
    create_unsafe_preflight_table(&database_url).await?;

    let capture = PgCapture::connect(PgCaptureConfig {
        connection_uri: database_url.clone(),
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: "unused_preflight_publication".to_string(),
        slot_name: "unused_preflight_slot".to_string(),
        tables: vec![
            TableSelector::new("public", "sales"),
            TableSelector::new("public", "trellara_preflight_default_pk"),
            TableSelector::new("public", "trellara_preflight_no_pk"),
            TableSelector::new("public", "trellara_preflight_missing"),
        ],
        create_if_missing: false,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    let preflight = capture.inspect_tables().await?;

    let sales = preflight
        .iter()
        .find(|table| table.name == "sales")
        .expect("sales preflight");
    assert!(sales.exists);
    assert_eq!(sales.replica_identity, Some(ReplicaIdentity::Full));
    assert!(sales.update_delete_safe);
    assert!(sales.issues.is_empty());
    assert!(sales.schema_fingerprint.is_some());
    assert!(sales.columns.iter().any(|column| {
        column.name == "id" && column.type_name == "text" && column.is_key && !column.nullable
    }));
    assert!(sales
        .columns
        .iter()
        .any(|column| column.name == "amount_cents" && column.type_oid > 0));

    let default_pk = preflight
        .iter()
        .find(|table| table.name == "trellara_preflight_default_pk")
        .expect("default-pk preflight");
    assert!(default_pk.exists);
    assert_eq!(default_pk.replica_identity, Some(ReplicaIdentity::Default));
    assert!(default_pk.update_delete_safe);
    assert!(default_pk.issues.is_empty());
    assert!(default_pk
        .contract_notes
        .iter()
        .any(|note| note.contains("REPLICA IDENTITY FULL is not required")));
    assert!(default_pk
        .contract_notes
        .iter()
        .any(|note| note.contains("unchanged TOAST values")));

    let no_pk = preflight
        .iter()
        .find(|table| table.name == "trellara_preflight_no_pk")
        .expect("no-pk preflight");
    assert!(no_pk.exists);
    assert_eq!(no_pk.replica_identity, Some(ReplicaIdentity::Default));
    assert!(!no_pk.update_delete_safe);
    assert_eq!(no_pk.issues.len(), 1);
    assert!(no_pk.schema_fingerprint.is_some());
    assert_eq!(
        no_pk
            .columns
            .iter()
            .map(|column| column.name.as_str())
            .collect::<Vec<_>>(),
        vec!["id", "value"]
    );

    let missing = preflight
        .iter()
        .find(|table| table.name == "trellara_preflight_missing")
        .expect("missing preflight");
    assert!(!missing.exists);
    assert_eq!(missing.issues, vec!["table does not exist"]);
    assert!(missing.columns.is_empty());
    assert!(missing.schema_fingerprint.is_none());

    assert!(matches!(
        capture.ensure_capture_safe().await,
        Err(CaptureError::PreflightFailed { issues }) if issues.len() == 2
    ));

    reset_preflight_table(&database_url).await?;
    Ok(())
}

#[tokio::test]
async fn capture_inspects_subscription_conflict_stats_when_available() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };

    let capture = PgCapture::connect(PgCaptureConfig {
        connection_uri: database_url,
        source_id: "source-integration".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "retail-sales".to_string(),
        publication_name: "unused_subscription_stats_publication".to_string(),
        slot_name: "unused_subscription_stats_slot".to_string(),
        tables: vec![TableSelector::new("public", "sales")],
        create_if_missing: false,
        stream_spill_threshold_changes: trellara_pg_capture::DEFAULT_STREAM_SPILL_THRESHOLD_CHANGES,
        stream_spill_dir: None,
        pgoutput: Default::default(),
    })
    .await?;

    let stats = capture.inspect_subscription_conflict_stats().await?;
    assert!(stats.iter().all(|stat| stat.total_conflicts() >= 0));
    Ok(())
}
