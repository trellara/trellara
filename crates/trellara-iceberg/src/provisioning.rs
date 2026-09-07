use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::{
    plan_raw_cdc_iceberg_table_specs, IcebergCommitConfig, IcebergRawCdcTableProvisioningPlan,
    IcebergTableProvisioningAction, Result,
};

#[cfg(feature = "iceberg-rust")]
mod compatibility;
#[cfg(feature = "iceberg-rust")]
mod runtime;
#[cfg(feature = "iceberg-rust")]
mod schema;
#[cfg(feature = "iceberg-rust")]
mod support;

#[cfg(feature = "iceberg-rust")]
use crate::{
    IcebergDdlAcknowledgement, IcebergMetadataTableSpec, IcebergRawCdcColumn,
    IcebergRawCdcPartitionField, IcebergTableIdentifier, IcebergTableProvisioningOutcome,
};
#[cfg(feature = "iceberg-rust")]
use apache_iceberg::Catalog;
#[cfg(feature = "iceberg-rust")]
use runtime::provision_tables;

#[cfg(feature = "iceberg-rust")]
const SCHEMA_FINGERPRINT_PROPERTY: &str = "trellara.schema-fingerprint-sha256";
#[cfg(feature = "iceberg-rust")]
const DDL_BARRIER_PROPERTY: &str = "trellara.ddl-barrier-id";
#[cfg(feature = "iceberg-rust")]
const DDL_ACK_LSN_PROPERTY: &str = "trellara.ddl-ack-lsn";
#[cfg(feature = "iceberg-rust")]
const DDL_SCHEMA_VERSION_PROPERTY: &str = "trellara.ddl-schema-version";

pub fn plan_raw_cdc_iceberg_table_provisioning(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
) -> Result<Vec<IcebergRawCdcTableProvisioningPlan>> {
    plan_raw_cdc_iceberg_table_specs(write_plan, config)?
        .into_iter()
        .map(|spec| {
            Ok(IcebergRawCdcTableProvisioningPlan {
                lake_table_name: spec.lake_table_name,
                relation: spec.relation,
                target: spec.target,
                schema_fingerprint_sha256: spec.schema_fingerprint_sha256,
                actions: vec![
                    IcebergTableProvisioningAction::EnsureNamespace,
                    IcebergTableProvisioningAction::CreateTableIfMissing,
                    IcebergTableProvisioningAction::VerifyCompatible,
                    IcebergTableProvisioningAction::EvolveSchemaAfterDdlAck,
                ],
                columns: spec.columns,
                partition_fields: spec.partition_fields,
                write_mode: "append_only_raw_cdc".to_string(),
                schema_evolution_gate:
                    "only apply schema changes after matching Trellara DDL acknowledgement evidence"
                        .to_string(),
                partition_evolution_gate:
                    "do not change the epoch/source_bucket partition contract without DDL acknowledgement and pending-epoch protection"
                        .to_string(),
            })
        })
        .collect()
}

#[cfg(feature = "iceberg-rust")]
pub async fn provision_raw_cdc_iceberg_tables(
    catalog: &dyn Catalog,
    dataset_id: &str,
    plans: &[IcebergRawCdcTableProvisioningPlan],
    ddl_ack: Option<&IcebergDdlAcknowledgement>,
) -> Result<Vec<IcebergTableProvisioningOutcome>> {
    let desired = plans
        .iter()
        .map(|plan| DesiredTable {
            target: &plan.target,
            schema_fingerprint: &plan.schema_fingerprint_sha256,
            columns: &plan.columns,
            partition_fields: &plan.partition_fields,
        })
        .collect::<Vec<_>>();
    provision_tables(catalog, dataset_id, &desired, ddl_ack).await
}

#[cfg(feature = "iceberg-rust")]
pub async fn provision_iceberg_metadata_tables(
    catalog: &dyn Catalog,
    dataset_id: &str,
    specs: &[IcebergMetadataTableSpec],
    ddl_ack: Option<&IcebergDdlAcknowledgement>,
) -> Result<Vec<IcebergTableProvisioningOutcome>> {
    let desired = specs
        .iter()
        .map(|spec| DesiredTable {
            target: &spec.target,
            schema_fingerprint: &spec.schema_fingerprint_sha256,
            columns: &spec.columns,
            partition_fields: &spec.partition_fields,
        })
        .collect::<Vec<_>>();
    provision_tables(catalog, dataset_id, &desired, ddl_ack).await
}

#[cfg(feature = "iceberg-rust")]
pub(super) struct DesiredTable<'a> {
    pub(super) target: &'a IcebergTableIdentifier,
    pub(super) schema_fingerprint: &'a str,
    pub(super) columns: &'a [IcebergRawCdcColumn],
    pub(super) partition_fields: &'a [IcebergRawCdcPartitionField],
}
