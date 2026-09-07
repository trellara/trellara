use apache_iceberg::Catalog;
use futures::{stream, StreamExt, TryStreamExt};
use trellara_checkpoint::IcebergCommitStore;
use trellara_lake::LakeRawCdcEpochWritePlan;

mod types;
pub use types::{ProductionIcebergCommitTimes, ProductionIcebergEpochResult};

use crate::{
    commit_iceberg_epoch_with_checkpoint, commit_iceberg_table_append,
    encode_iceberg_metadata_files, iceberg_epoch_commit_summary, plan_iceberg_epoch_commit,
    plan_iceberg_metadata_commit_bundle, plan_iceberg_metadata_table_specs,
    plan_raw_cdc_iceberg_table_provisioning, provision_iceberg_metadata_tables,
    provision_raw_cdc_iceberg_tables, upload_iceberg_metadata_file, write_and_upload_raw_cdc_file,
    IcebergCommitConfig, IcebergDdlAcknowledgement, IcebergObjectStore, Result,
};

const MAX_PARALLEL_OBJECT_UPLOADS: usize = 8;

/// Execute the production epoch protocol. The checkpoint intent for each table
/// is persisted before its catalog commit; `_trellara_epochs` is committed only
/// after every raw and supporting metadata receipt has been durably recorded.
pub async fn write_production_iceberg_epoch<S, K>(
    catalog: &dyn Catalog,
    object_store: &S,
    checkpoint: &K,
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
    ddl_ack: Option<&IcebergDdlAcknowledgement>,
    commit_times: ProductionIcebergCommitTimes,
) -> Result<ProductionIcebergEpochResult>
where
    S: IcebergObjectStore + ?Sized,
    K: IcebergCommitStore,
{
    let planned_at = commit_times.planned_at;
    let committed_at = commit_times.committed_at;
    let s3 = config.object_store.as_ref().ok_or_else(|| {
        crate::IcebergIntegrationError::InvalidObjectStoreConfig {
            field: "object_store",
            reason: "production writer requires an S3 object-store configuration".to_string(),
        }
    })?;
    let raw_provisioning = plan_raw_cdc_iceberg_table_provisioning(write_plan, config)?;
    let metadata_specs = plan_iceberg_metadata_table_specs(write_plan, config)?;
    let raw_table_provisioning = provision_raw_cdc_iceberg_tables(
        catalog,
        &write_plan.dataset_id,
        &raw_provisioning,
        ddl_ack,
    )
    .await?;
    let metadata_table_provisioning = provision_iceberg_metadata_tables(
        catalog,
        &write_plan.dataset_id,
        &metadata_specs,
        ddl_ack,
    )
    .await?;

    let mut raw_uploads = stream::iter(write_plan.data_files.iter().enumerate().map(
        |(index, data_file)| async move {
            write_and_upload_raw_cdc_file(object_store, s3, write_plan, data_file)
                .await
                .map(|upload| (index, upload))
        },
    ))
    .buffer_unordered(MAX_PARALLEL_OBJECT_UPLOADS)
    .try_collect::<Vec<_>>()
    .await?;
    raw_uploads.sort_by_key(|(index, _)| *index);
    let raw_uploads = raw_uploads
        .into_iter()
        .map(|(_, upload)| upload)
        .collect::<Vec<_>>();
    let raw_commit_plan = plan_iceberg_epoch_commit(
        write_plan,
        config,
        raw_uploads
            .iter()
            .map(|upload| upload.completed_file.clone())
            .collect(),
    )?;
    let raw_receipts = commit_iceberg_epoch_with_checkpoint(
        checkpoint,
        &raw_commit_plan,
        &planned_at,
        &committed_at,
        |table| async move { commit_iceberg_table_append(catalog, &table).await },
    )
    .await?;
    let raw_summary = iceberg_epoch_commit_summary(&raw_commit_plan, &raw_receipts)?;
    let epoch_snapshot_reference = raw_summary.epoch_snapshot_reference.ok_or_else(|| {
        crate::IcebergIntegrationError::EpochMetadataNotReady {
            epoch_id: write_plan.epoch_id.clone(),
            missing_tables: raw_summary.missing_tables,
        }
    })?;
    let encoded_metadata = encode_iceberg_metadata_files(
        write_plan,
        &metadata_specs,
        &epoch_snapshot_reference,
        &raw_summary.snapshot_ids,
    )?;
    let mut metadata_uploads = stream::iter(encoded_metadata.into_iter().enumerate().map(
        |(index, encoded)| async move {
            upload_iceberg_metadata_file(object_store, s3, encoded)
                .await
                .map(|upload| (index, upload))
        },
    ))
    .buffer_unordered(MAX_PARALLEL_OBJECT_UPLOADS)
    .try_collect::<Vec<_>>()
    .await?;
    metadata_uploads.sort_by_key(|(index, _)| *index);
    let metadata_uploads = metadata_uploads
        .into_iter()
        .map(|(_, upload)| upload)
        .collect::<Vec<_>>();
    let metadata_commit_bundle = plan_iceberg_metadata_commit_bundle(
        write_plan,
        &raw_commit_plan,
        &metadata_specs,
        raw_summary.snapshot_ids,
        epoch_snapshot_reference,
        metadata_uploads.clone(),
    )?;
    let supporting_metadata_receipts = commit_iceberg_epoch_with_checkpoint(
        checkpoint,
        &metadata_commit_bundle.supporting_metadata_commit_plan,
        &planned_at,
        &committed_at,
        |table| async move { commit_iceberg_table_append(catalog, &table).await },
    )
    .await?;
    iceberg_epoch_commit_summary(
        &metadata_commit_bundle.supporting_metadata_commit_plan,
        &supporting_metadata_receipts,
    )?;
    let completeness_receipts = commit_iceberg_epoch_with_checkpoint(
        checkpoint,
        &metadata_commit_bundle.completeness_commit_plan,
        &planned_at,
        &committed_at,
        |table| async move { commit_iceberg_table_append(catalog, &table).await },
    )
    .await?;
    iceberg_epoch_commit_summary(
        &metadata_commit_bundle.completeness_commit_plan,
        &completeness_receipts,
    )?;

    Ok(ProductionIcebergEpochResult {
        raw_table_provisioning,
        metadata_table_provisioning,
        raw_uploads,
        raw_commit_plan,
        raw_receipts,
        metadata_uploads,
        metadata_commit_bundle,
        supporting_metadata_receipts,
        completeness_receipts,
    })
}
