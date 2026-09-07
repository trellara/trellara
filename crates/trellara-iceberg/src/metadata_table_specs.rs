use serde::{Deserialize, Serialize};
use trellara_lake::LakeRawCdcEpochWritePlan;

mod columns;
use columns::{
    completeness_columns, partition_columns, quarantine_columns, source_columns, table_columns,
    verification_columns,
};

use crate::epoch_metadata_validation::metadata_target;
use crate::raw_cdc_schema::schema_fingerprint;
use crate::{
    IcebergCommitConfig, IcebergRawCdcColumn, IcebergRawCdcPartitionField, IcebergTableIdentifier,
    Result,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IcebergMetadataTableKind {
    Source,
    Table,
    Partition,
    Quarantine,
    Verification,
    Completeness,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IcebergMetadataTableSpec {
    pub kind: IcebergMetadataTableKind,
    pub lake_table_name: String,
    pub target: IcebergTableIdentifier,
    pub schema_fingerprint_sha256: String,
    pub columns: Vec<IcebergRawCdcColumn>,
    pub partition_fields: Vec<IcebergRawCdcPartitionField>,
    pub commit_order: u8,
    pub visibility_rule: String,
}

pub fn plan_iceberg_metadata_table_specs(
    write_plan: &LakeRawCdcEpochWritePlan,
    config: &IcebergCommitConfig,
) -> Result<Vec<IcebergMetadataTableSpec>> {
    let epochs_target = metadata_target(write_plan, config)?;
    let namespace = epochs_target.namespace.clone();
    let metadata = &write_plan.epoch_metadata;
    Ok(vec![
        metadata_spec(
            IcebergMetadataTableKind::Source,
            &metadata.epoch_sources_table,
            &namespace,
            "_trellara_epoch_sources",
            source_columns(),
            10,
        )?,
        metadata_spec(
            IcebergMetadataTableKind::Table,
            &metadata.epoch_tables_table,
            &namespace,
            "_trellara_epoch_tables",
            table_columns(),
            20,
        )?,
        metadata_spec(
            IcebergMetadataTableKind::Partition,
            &metadata.epoch_partitions_table,
            &namespace,
            "_trellara_epoch_partitions",
            partition_columns(),
            30,
        )?,
        metadata_spec(
            IcebergMetadataTableKind::Quarantine,
            &metadata.quarantine_table,
            &namespace,
            "_trellara_quarantine",
            quarantine_columns(),
            40,
        )?,
        metadata_spec(
            IcebergMetadataTableKind::Verification,
            &metadata.verification_table,
            &namespace,
            "_trellara_verification",
            verification_columns(),
            50,
        )?,
        metadata_spec(
            IcebergMetadataTableKind::Completeness,
            &metadata.epochs_table,
            &namespace,
            &epochs_target.name,
            completeness_columns(),
            100,
        )?,
    ])
}

fn metadata_spec(
    kind: IcebergMetadataTableKind,
    lake_table_name: &str,
    namespace: &[String],
    target_name: &str,
    columns: Vec<IcebergRawCdcColumn>,
    commit_order: u8,
) -> Result<IcebergMetadataTableSpec> {
    let target = IcebergTableIdentifier::new(namespace.iter().cloned(), target_name)?;
    let partition_fields = Vec::new();
    Ok(IcebergMetadataTableSpec {
        kind,
        lake_table_name: lake_table_name.to_string(),
        target,
        schema_fingerprint_sha256: schema_fingerprint(&columns, &partition_fields),
        columns,
        partition_fields,
        commit_order,
        visibility_rule: match kind {
            IcebergMetadataTableKind::Completeness => "commit last; this row is the SQL-visible release marker after raw and supporting metadata snapshots validate".to_string(),
            _ => "commit after raw table snapshots validate and before the completeness release marker".to_string(),
        },
    })
}
