use std::collections::HashMap;

use apache_iceberg::spec::{
    DataContentType, DataFile, DataFileBuilder, DataFileFormat, Literal, Struct, Transform,
};
use apache_iceberg::TableIdent;

use crate::{
    IcebergCompletedDataFile, IcebergIntegrationError, IcebergTableAppendPlan,
    IcebergTableCommitReceipt, IcebergTableCommitStatus, Result, SNAPSHOT_PROPERTY_EPOCH_COMMIT_ID,
    SNAPSHOT_PROPERTY_TABLE_COMMIT_ID,
};

pub(crate) fn table_ident(plan: &IcebergTableAppendPlan) -> Result<TableIdent> {
    let mut components = plan.target.namespace.clone();
    components.push(plan.target.name.clone());
    TableIdent::from_strs(components).map_err(|error| {
        IcebergIntegrationError::InvalidTableIdentifier {
            identifier: plan.target.qualified_name(),
            reason: error.to_string(),
        }
    })
}

pub(crate) fn data_file(
    table: &apache_iceberg::table::Table,
    plan: &IcebergTableAppendPlan,
    file: &IcebergCompletedDataFile,
) -> Result<DataFile> {
    DataFileBuilder::default()
        .content(DataContentType::Data)
        .file_path(file.file_uri.clone())
        .file_format(DataFileFormat::Parquet)
        .partition(partition_tuple(table, plan, file)?)
        .partition_spec_id(table.metadata().default_partition_spec_id())
        .file_size_in_bytes(file.file_size_in_bytes)
        .record_count(file.record_count)
        .build()
        .map_err(|error| IcebergIntegrationError::Catalog {
            operation: "build_data_file",
            target: table.identifier().to_string(),
            message: error.to_string(),
        })
}

fn partition_tuple(
    table: &apache_iceberg::table::Table,
    plan: &IcebergTableAppendPlan,
    file: &IcebergCompletedDataFile,
) -> Result<Struct> {
    let spec = table.metadata().default_partition_spec();
    if spec.is_unpartitioned() {
        return Ok(Struct::empty());
    }

    let fields = spec.fields();
    if fields.len() != 2 {
        return unsupported_partition(plan, "expected exactly epoch_id and source_bucket fields");
    }
    let epoch_field = &fields[0];
    let source_bucket_field = &fields[1];
    if epoch_field.name != "epoch_id"
        || epoch_field.source_id != 26
        || epoch_field.transform != Transform::Identity
    {
        return unsupported_partition(
            plan,
            "first partition field must be identity(epoch_id) with source id 26",
        );
    }
    if source_bucket_field.name != "source_bucket"
        || source_bucket_field.source_id != 2
        || source_bucket_field.transform != Transform::Identity
    {
        return unsupported_partition(
            plan,
            "second partition field must be identity(source_bucket) with source id 2",
        );
    }
    let source_bucket =
        i32::try_from(file.source_bucket).map_err(|_| IcebergIntegrationError::CountOverflow {
            field: "source_bucket",
        })?;
    Ok(Struct::from_iter([
        Some(Literal::string(
            plan.snapshot_properties[crate::SNAPSHOT_PROPERTY_EPOCH_ID].clone(),
        )),
        Some(Literal::int(source_bucket)),
    ]))
}

fn unsupported_partition<T>(plan: &IcebergTableAppendPlan, reason: &str) -> Result<T> {
    Err(IcebergIntegrationError::UnsupportedPartitionSpec {
        target: plan.target.qualified_name(),
        reason: reason.to_string(),
    })
}

pub(crate) fn matching_snapshot_id(
    table: &apache_iceberg::table::Table,
    plan: &IcebergTableAppendPlan,
) -> Result<Option<i64>> {
    let matching = table
        .metadata()
        .snapshots()
        .filter(|snapshot| {
            snapshot
                .summary()
                .additional_properties
                .get(SNAPSHOT_PROPERTY_TABLE_COMMIT_ID)
                == Some(&plan.table_commit_id)
        })
        .collect::<Vec<_>>();
    if matching.len() > 1 {
        return Err(IcebergIntegrationError::ConflictingCatalogEvidence {
            target: plan.target.qualified_name(),
            reason: "table commit id appears in more than one snapshot".to_string(),
        });
    }
    let Some(snapshot) = matching.first() else {
        return Ok(None);
    };
    validate_snapshot_properties(plan, &snapshot.summary().additional_properties)?;
    Ok(Some(snapshot.snapshot_id()))
}

pub(crate) fn validate_snapshot_properties(
    plan: &IcebergTableAppendPlan,
    properties: &HashMap<String, String>,
) -> Result<()> {
    for (key, expected) in &plan.snapshot_properties {
        let Some(actual) = properties.get(key) else {
            return Err(IcebergIntegrationError::ConflictingCatalogEvidence {
                target: plan.target.qualified_name(),
                reason: format!("snapshot with matching commit id is missing property {key}"),
            });
        };
        if actual != expected {
            return Err(IcebergIntegrationError::ConflictingCatalogEvidence {
                target: plan.target.qualified_name(),
                reason: format!(
                    "snapshot property {key} mismatch: expected {expected}, found {actual}"
                ),
            });
        }
    }
    if properties.get(SNAPSHOT_PROPERTY_EPOCH_COMMIT_ID) != Some(&plan.epoch_commit_id) {
        return Err(IcebergIntegrationError::ConflictingCatalogEvidence {
            target: plan.target.qualified_name(),
            reason: "snapshot epoch commit id does not match the table append plan".to_string(),
        });
    }
    Ok(())
}

pub(crate) fn receipt(
    plan: &IcebergTableAppendPlan,
    snapshot_id: i64,
    status: IcebergTableCommitStatus,
) -> IcebergTableCommitReceipt {
    IcebergTableCommitReceipt {
        target: plan.target.clone(),
        epoch_commit_id: plan.epoch_commit_id.clone(),
        table_commit_id: plan.table_commit_id.clone(),
        snapshot_id,
        file_count: plan.file_count,
        record_count: plan.record_count,
        status,
    }
}
