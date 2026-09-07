use super::*;

pub(crate) async fn reset_target_table(client: &tokio_postgres::Client) -> TestResult<()> {
    client
        .batch_execute(&format!(
            r#"
            create table if not exists public.{TABLE_NAME} (
                id text primary key,
                amount_cents text not null,
                reviewed_by text not null default 'target-default',
                receipt_blob text not null default 'target-receipt',
                receipt_bytes bytea not null default '\x'::bytea
            );
            alter table public.{TABLE_NAME}
                add column if not exists reviewed_by text not null default 'target-default';
            alter table public.{TABLE_NAME}
                add column if not exists receipt_blob text not null default 'target-receipt';
            alter table public.{TABLE_NAME}
                add column if not exists receipt_bytes bytea not null default '\x'::bytea;
            truncate table public.{TABLE_NAME};
            delete from trellara.applied_transactions
             where source_id = '{SOURCE_ID}'
               and dataset_id = '{DATASET_ID}';
            delete from trellara.flow_checkpoints
             where source_id = '{SOURCE_ID}'
               and dataset_id = '{DATASET_ID}';
            delete from trellara.apply_quarantine
             where source_id = '{SOURCE_ID}'
               and dataset_id = '{DATASET_ID}';
            delete from trellara.partition_checkpoints
             where source_id = '{SOURCE_ID}'
               and dataset_id = '{DATASET_ID}';
            "#
        ))
        .await?;
    Ok(())
}
