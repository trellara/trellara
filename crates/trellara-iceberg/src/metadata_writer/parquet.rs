use std::collections::HashMap;
use std::sync::Arc;

use arrow_array::{ArrayRef, Int32Array, Int64Array, RecordBatch, StringArray};
use arrow_schema::{DataType, Field, Schema};
use bytes::Bytes;
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::{WriterProperties, WriterVersion};
use trellara_lake::LakeRawCdcEpochWritePlan;

use super::values::MetadataValue;
use super::IcebergEncodedMetadataFile;
use crate::{IcebergIntegrationError, IcebergMetadataTableSpec, IcebergRawCdcColumnType, Result};

pub(super) fn encode_metadata_file(
    write_plan: &LakeRawCdcEpochWritePlan,
    spec: &IcebergMetadataTableSpec,
    rows: Vec<Vec<MetadataValue>>,
) -> Result<IcebergEncodedMetadataFile> {
    let schema = Arc::new(Schema::new(
        spec.columns
            .iter()
            .map(|column| {
                Field::new(
                    &column.name,
                    data_type(column.column_type),
                    !column.required,
                )
                .with_metadata(HashMap::from([(
                    parquet::arrow::PARQUET_FIELD_ID_META_KEY.to_string(),
                    column.id.to_string(),
                )]))
            })
            .collect::<Vec<_>>(),
    ));
    if rows.iter().any(|row| row.len() != spec.columns.len()) {
        return metadata_error(spec, "row width differs from the table specification");
    }
    let arrays = spec
        .columns
        .iter()
        .enumerate()
        .map(|(index, column)| metadata_array(spec, column.column_type, &rows, index))
        .collect::<Result<Vec<_>>>()?;
    let batch = RecordBatch::try_new(schema, arrays).map_err(|error| error_value(spec, error))?;
    let properties = WriterProperties::builder()
        .set_writer_version(WriterVersion::PARQUET_2_0)
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let mut writer = ArrowWriter::try_new(Vec::new(), batch.schema(), Some(properties))
        .map_err(|error| error_value(spec, error))?;
    writer
        .write(&batch)
        .map_err(|error| error_value(spec, error))?;
    let content = writer
        .into_inner()
        .map_err(|error| error_value(spec, error))?;
    let record_count =
        u64::try_from(rows.len()).map_err(|_| IcebergIntegrationError::CountOverflow {
            field: "metadata_record_count",
        })?;
    Ok(IcebergEncodedMetadataFile {
        kind: spec.kind,
        planned_object_key: format!(
            "{}/epoch_id={}/part-00000.parquet",
            spec.lake_table_name, write_plan.epoch_id
        ),
        table_name: spec.lake_table_name.clone(),
        relation: format!("trellara.fanin.{:?}", spec.kind).to_lowercase(),
        content: Bytes::from(content),
        record_count,
        checksum_rollup: write_plan.checksum_rollup,
    })
}

fn data_type(column_type: IcebergRawCdcColumnType) -> DataType {
    match column_type {
        IcebergRawCdcColumnType::Int => DataType::Int32,
        IcebergRawCdcColumnType::Long => DataType::Int64,
        IcebergRawCdcColumnType::String => DataType::Utf8,
    }
}

fn metadata_array(
    spec: &IcebergMetadataTableSpec,
    column_type: IcebergRawCdcColumnType,
    rows: &[Vec<MetadataValue>],
    index: usize,
) -> Result<ArrayRef> {
    match column_type {
        IcebergRawCdcColumnType::String => rows
            .iter()
            .map(|row| match &row[index] {
                MetadataValue::String(value) => Ok(value.clone()),
                _ => metadata_error(spec, "metadata value type differs from schema"),
            })
            .collect::<Result<Vec<_>>>()
            .map(|values| Arc::new(StringArray::from(values)) as ArrayRef),
        IcebergRawCdcColumnType::Int => rows
            .iter()
            .map(|row| match row[index] {
                MetadataValue::Int(value) => Ok(value),
                _ => metadata_error(spec, "metadata value type differs from schema"),
            })
            .collect::<Result<Vec<_>>>()
            .map(|values| Arc::new(Int32Array::from(values)) as ArrayRef),
        IcebergRawCdcColumnType::Long => rows
            .iter()
            .map(|row| match row[index] {
                MetadataValue::Long(value) => Ok(value),
                _ => metadata_error(spec, "metadata value type differs from schema"),
            })
            .collect::<Result<Vec<_>>>()
            .map(|values| Arc::new(Int64Array::from(values)) as ArrayRef),
    }
}

fn metadata_error<T>(spec: &IcebergMetadataTableSpec, message: &str) -> Result<T> {
    Err(IcebergIntegrationError::MetadataEncoding {
        table: spec.target.qualified_name(),
        message: message.to_string(),
    })
}

fn error_value(
    spec: &IcebergMetadataTableSpec,
    error: impl std::fmt::Display,
) -> IcebergIntegrationError {
    IcebergIntegrationError::MetadataEncoding {
        table: spec.target.qualified_name(),
        message: error.to_string(),
    }
}
