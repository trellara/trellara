use std::collections::HashMap;
use std::sync::Arc;

use apache_iceberg::spec::{
    NestedField, PrimitiveType, Schema, Transform, Type, UnboundPartitionSpec,
};
use apache_iceberg::TableCreation;

use super::support::{catalog_error, incompatible};
use super::{
    DesiredTable, DDL_ACK_LSN_PROPERTY, DDL_BARRIER_PROPERTY, DDL_SCHEMA_VERSION_PROPERTY,
    SCHEMA_FINGERPRINT_PROPERTY,
};
use crate::{IcebergDdlAcknowledgement, IcebergIntegrationError, IcebergRawCdcColumnType, Result};

pub(super) fn require_ddl_ack<'a>(
    dataset_id: &str,
    desired: &DesiredTable<'_>,
    ddl_ack: Option<&'a IcebergDdlAcknowledgement>,
    operation: &str,
) -> Result<&'a IcebergDdlAcknowledgement> {
    let Some(ack) = ddl_ack else {
        return Err(IcebergIntegrationError::DdlAcknowledgementRequired {
            target: desired.target.qualified_name(),
            reason: format!("{operation} requires acknowledgement bound to the desired schema"),
        });
    };
    if !ack.authorizes(dataset_id, desired.schema_fingerprint) {
        return Err(IcebergIntegrationError::DdlAcknowledgementRequired {
            target: desired.target.qualified_name(),
            reason: format!("acknowledgement dataset/fingerprint does not authorize {operation}"),
        });
    }
    Ok(ack)
}

pub(super) fn table_creation(
    desired: &DesiredTable<'_>,
    ack: &IcebergDdlAcknowledgement,
) -> Result<TableCreation> {
    let partition_spec = if desired.partition_fields.is_empty() {
        None
    } else {
        Some(partition_spec(desired)?)
    };
    Ok(TableCreation::builder()
        .name(desired.target.name.clone())
        .schema(iceberg_schema(desired)?)
        .partition_spec_opt(partition_spec)
        .properties(schema_properties(desired, ack))
        .build())
}

pub(super) fn iceberg_type(column_type: IcebergRawCdcColumnType) -> Type {
    Type::Primitive(match column_type {
        IcebergRawCdcColumnType::Int => PrimitiveType::Int,
        IcebergRawCdcColumnType::Long => PrimitiveType::Long,
        IcebergRawCdcColumnType::String => PrimitiveType::String,
    })
}

fn iceberg_schema(desired: &DesiredTable<'_>) -> Result<Schema> {
    Schema::builder()
        .with_fields(desired.columns.iter().map(|column| {
            Arc::new(NestedField::new(
                column.id,
                &column.name,
                iceberg_type(column.column_type),
                column.required,
            ))
        }))
        .build()
        .map_err(|error| catalog_error("build_schema", desired.target, error))
}

fn partition_spec(desired: &DesiredTable<'_>) -> Result<UnboundPartitionSpec> {
    let mut builder = UnboundPartitionSpec::builder();
    for field in desired.partition_fields {
        if field.transform != "identity" {
            return incompatible(
                desired,
                format!("unsupported partition transform {}", field.transform),
            );
        }
        builder = builder
            .add_partition_field(
                field.source_column_id,
                &field.source_column,
                Transform::Identity,
            )
            .map_err(|error| catalog_error("build_partition_spec", desired.target, error))?;
    }
    Ok(builder.build())
}

fn schema_properties(
    desired: &DesiredTable<'_>,
    ack: &IcebergDdlAcknowledgement,
) -> HashMap<String, String> {
    HashMap::from([
        (
            SCHEMA_FINGERPRINT_PROPERTY.to_string(),
            desired.schema_fingerprint.to_string(),
        ),
        (DDL_BARRIER_PROPERTY.to_string(), ack.barrier_id.clone()),
        (DDL_ACK_LSN_PROPERTY.to_string(), ack.ack_lsn.clone()),
        (
            DDL_SCHEMA_VERSION_PROPERTY.to_string(),
            ack.schema_version.clone(),
        ),
    ])
}
