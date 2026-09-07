use std::collections::BTreeMap;
use std::io::Write;

use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::{WriterProperties, WriterVersion};

use super::epoch_metadata_batch::record_batch;
use super::epoch_metadata_schema::raw_cdc_epoch_metadata_parquet_arrow_schema as epoch_metadata_schema;
use super::epoch_metadata_validation::validate_snapshot_evidence;
use super::hash::HashingWriter;
use super::parquet_error;
use crate::raw_cdc_naming::epoch_metadata_object_key_hint;
use crate::{
    ensure_lake_epoch_consumable, LakeEpochConsumerOptions, LakeError,
    LakeRawCdcEpochMetadataParquetWriteEvidence, LakeRawCdcEpochWritePlan,
};

pub struct LakeRawCdcEpochMetadataParquetWriteOutput<W> {
    pub writer: W,
    pub evidence: LakeRawCdcEpochMetadataParquetWriteEvidence,
}

pub fn write_raw_cdc_epoch_metadata_parquet_file<W: Write + Send>(
    writer: W,
    write_plan: &LakeRawCdcEpochWritePlan,
    iceberg_snapshot_id: impl Into<String>,
    raw_table_snapshot_ids: BTreeMap<String, i64>,
    file_uri: impl Into<String>,
    object_version: Option<String>,
) -> Result<LakeRawCdcEpochMetadataParquetWriteOutput<W>, LakeError> {
    let iceberg_snapshot_id = iceberg_snapshot_id.into();
    validate_snapshot_evidence(&iceberg_snapshot_id, &raw_table_snapshot_ids)?;
    ensure_lake_epoch_consumable(
        &write_plan.epoch_metadata.epoch_row,
        &write_plan.epoch_metadata.verification_row,
        LakeEpochConsumerOptions::accepting_complete_with_gaps(),
    )?;
    let batch = record_batch(write_plan, &iceberg_snapshot_id, &raw_table_snapshot_ids)?;
    let properties = WriterProperties::builder()
        .set_writer_version(WriterVersion::PARQUET_2_0)
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let hashing_writer = HashingWriter::new(writer);
    let mut writer = ArrowWriter::try_new(hashing_writer, batch.schema(), Some(properties))
        .map_err(|error| parquet_error("create_epoch_metadata_arrow_writer", error))?;
    writer
        .write(&batch)
        .map_err(|error| parquet_error("write_epoch_metadata_record_batch", error))?;
    let hashing_writer = writer
        .into_inner()
        .map_err(|error| parquet_error("recover_epoch_metadata_writer", error))?;
    let (writer, file_size_in_bytes, content_sha256) = hashing_writer.finish();

    Ok(LakeRawCdcEpochMetadataParquetWriteOutput {
        writer,
        evidence: LakeRawCdcEpochMetadataParquetWriteEvidence {
            planned_object_key: epoch_metadata_object_key_hint(
                &write_plan.epoch_id,
                &write_plan.epoch_metadata.epochs_table,
            ),
            table_name: write_plan.epoch_metadata.epochs_table.clone(),
            epoch_id: write_plan.epoch_id.clone(),
            dataset_id: write_plan.dataset_id.clone(),
            file_uri: file_uri.into(),
            file_format: "parquet".to_string(),
            file_size_in_bytes,
            content_sha256,
            object_version,
            record_count: 1,
            checksum_rollup: write_plan.checksum_rollup,
            manifest_digest: write_plan.epoch_metadata.epoch_row.manifest_digest.clone(),
            iceberg_snapshot_id,
            raw_table_snapshot_ids,
        },
    })
}

pub fn raw_cdc_epoch_metadata_parquet_arrow_schema() -> arrow_schema::SchemaRef {
    epoch_metadata_schema()
}
