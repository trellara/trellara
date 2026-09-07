use super::support::*;
use trellara_apply_postgres::{
    apply_target_ddl_envelope_then_dml, TargetDdlAckContext, TargetDdlApplyPlan,
    TargetDdlBarrierRequirements, TargetDdlStatement,
};
use trellara_checkpoint::{DdlBarrier, DdlBarrierLookup, DdlBarrierStore, InMemoryCheckpointStore};
use trellara_protocol::DdlEvent;

#[tokio::test]
async fn applies_target_ddl_plan_in_single_transaction() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    reset_target_table(&client).await?;
    client
        .batch_execute(&format!(
            "alter table public.{TABLE_NAME} drop column if exists ddl_marker"
        ))
        .await?;

    let outcome = applier
        .apply_target_ddl_plan(target_ddl_plan("ddl-barrier-integration", "ddl_marker"))
        .await?;

    assert_eq!(outcome.barrier_id, "ddl-barrier-integration");
    assert_eq!(outcome.applied_statements, 1);
    assert_eq!(outcome.release_gate, "post_ddl_dml_release");
    assert!(column_exists(&client, TABLE_NAME, "ddl_marker").await?);

    Ok(())
}

#[tokio::test]
async fn applies_target_ddl_plan_and_records_target_ack() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    let store = InMemoryCheckpointStore::new();
    let flow = DdlBarrierLookup::new("source-ddl-ack", "retail", "dataset-ddl-ack");
    reset_target_table(&client).await?;
    client
        .batch_execute(&format!(
            "alter table public.{TABLE_NAME} drop column if exists ddl_ack_marker"
        ))
        .await?;
    store
        .record_ddl_barrier(DdlBarrier {
            source_id: flow.source_id.clone(),
            database_id: flow.database_id.clone(),
            dataset_id: flow.dataset_id.clone(),
            barrier_id: "ddl-barrier-ack-integration".to_string(),
            barrier_lsn: "0/16B8000".to_string(),
            schema_version: "schema-v2".to_string(),
            cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
            required_sinks: vec!["target_postgres".to_string(), "raw_cdc_lake".to_string()],
            requires_global_partition_pause: false,
        })
        .await?;

    let evidence = applier
        .apply_target_ddl_plan_and_record_ack(
            target_ddl_plan("ddl-barrier-ack-integration", "ddl_ack_marker"),
            &store,
            TargetDdlAckContext::new(
                &flow.source_id,
                &flow.database_id,
                &flow.dataset_id,
                "0/16B9000",
                "schema-v2",
            ),
        )
        .await?;

    assert_eq!(evidence.sink, "target_postgres");
    assert!(column_exists(&client, TABLE_NAME, "ddl_ack_marker").await?);
    let summary = store
        .ddl_barrier_summary(&flow, "ddl-barrier-ack-integration")
        .await?
        .expect("barrier summary");
    assert_eq!(summary.acked_sinks, vec!["target_postgres"]);
    assert_eq!(summary.pending_sinks, vec!["raw_cdc_lake"]);
    assert!(!summary.release_dml);

    Ok(())
}

#[tokio::test]
async fn applies_target_ddl_from_envelope_then_dml_without_replaying_ddl() -> TestResult<()> {
    let _guard = INTEGRATION_LOCK.lock().await;
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let client = connect_test_client(&database_url).await?;
    let mut applier = connect_applier(&database_url).await?;
    let store = InMemoryCheckpointStore::new();
    let mut envelope = envelope(
        "tx-ddl-envelope-ack",
        "0/16B9F00",
        "0/16BA000",
        vec![insert_change(
            "tx-ddl-envelope-ack",
            2,
            "sale-ddl-envelope",
            "900",
        )],
    );
    reset_target_table(&client).await?;
    client
        .batch_execute(&format!(
            "alter table public.{TABLE_NAME} drop column if exists ddl_envelope_marker"
        ))
        .await?;
    envelope.ddl_events = vec![DdlEvent::additive_column(
        &envelope.transaction_id,
        2,
        relation(),
        format!("ALTER TABLE public.{TABLE_NAME} ADD COLUMN ddl_envelope_marker text;"),
        12_345,
        67_890,
    )];
    envelope.finalize_checksum();

    let outcome = apply_target_ddl_envelope_then_dml(
        &mut applier,
        &store,
        &envelope,
        TargetDdlBarrierRequirements::target_postgres_only(),
    )
    .await?;

    let ddl = outcome.ddl.expect("DDL outcome");
    assert_eq!(ddl.target_ack.barrier_id, ddl.barrier_summary.barrier_id);
    assert!(column_exists(&client, TABLE_NAME, "ddl_envelope_marker").await?);
    assert_eq!(
        sale_amount(&client, "sale-ddl-envelope").await?,
        Some("900".to_string())
    );
    assert!(ddl.barrier_summary.release_dml);
    assert_eq!(ddl.barrier_summary.acked_sinks, vec!["target_postgres"]);
    assert!(ddl.barrier_summary.pending_sinks.is_empty());
    assert_eq!(outcome.dml.applied_changes, 1);

    Ok(())
}

fn target_ddl_plan(barrier_id: &str, column: &str) -> TargetDdlApplyPlan {
    TargetDdlApplyPlan {
        barrier_id: barrier_id.to_string(),
        transaction_boundary_rule:
            "single_target_schema_transaction_then_barrier_ack_before_post_ddl_dml_release"
                .to_string(),
        blockers: Vec::new(),
        release_gate: "post_ddl_dml_release".to_string(),
        statements: vec![TargetDdlStatement {
            change: format!("add_nullable_column:public.trellara_apply_sales.{column}:text"),
            sql: format!("ALTER TABLE public.{TABLE_NAME} ADD COLUMN {column} text;"),
        }],
    }
}

async fn column_exists(
    client: &tokio_postgres::Client,
    table: &str,
    column: &str,
) -> TestResult<bool> {
    Ok(client
        .query_one(
            r#"
            select exists (
                select 1
                  from information_schema.columns
                 where table_schema = 'public'
                   and table_name = $1
                   and column_name = $2
            )
            "#,
            &[&table, &column],
        )
        .await?
        .get(0))
}
