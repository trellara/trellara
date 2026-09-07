use std::io::Write;

use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;

use crate::{
    LakeError, LakeRawCdcDataFilePlan, LakeRawCdcEpochWritePlan, LakeRawCdcParquetWriteEvidence,
};

mod batch;
mod epoch_metadata;
mod epoch_metadata_batch;
mod epoch_metadata_schema;
mod epoch_metadata_validation;
mod hash;
mod rows;
mod schema;

use batch::record_batch;
use hash::HashingWriter;
use rows::{rows_for_file, rows_len_u64};

pub use epoch_metadata::{
    raw_cdc_epoch_metadata_parquet_arrow_schema, write_raw_cdc_epoch_metadata_parquet_file,
    LakeRawCdcEpochMetadataParquetWriteOutput,
};
pub use schema::{raw_cdc_parquet_arrow_schema, RAW_CDC_PARQUET_FIELD_ID_KEY};

pub struct LakeRawCdcParquetWriteOutput<W> {
    pub writer: W,
    pub evidence: LakeRawCdcParquetWriteEvidence,
}

pub fn write_raw_cdc_parquet_file<W: Write + Send>(
    writer: W,
    write_plan: &LakeRawCdcEpochWritePlan,
    data_file: &LakeRawCdcDataFilePlan,
    file_uri: impl Into<String>,
    object_version: Option<String>,
) -> Result<LakeRawCdcParquetWriteOutput<W>, LakeError> {
    let rows = rows_for_file(write_plan, data_file)?;
    let record_count = rows_len_u64(rows.len())?;
    let batch = record_batch(&rows)?;
    let properties = WriterProperties::builder()
        .set_compression(Compression::ZSTD(Default::default()))
        .build();
    let hashing_writer = HashingWriter::new(writer);
    let mut writer = ArrowWriter::try_new(hashing_writer, batch.schema(), Some(properties))
        .map_err(|error| parquet_error("create_arrow_writer", error))?;
    writer
        .write(&batch)
        .map_err(|error| parquet_error("write_record_batch", error))?;
    let hashing_writer = writer
        .into_inner()
        .map_err(|error| parquet_error("recover_writer", error))?;
    let (writer, file_size_in_bytes, content_sha256) = hashing_writer.finish();

    Ok(LakeRawCdcParquetWriteOutput {
        writer,
        evidence: LakeRawCdcParquetWriteEvidence {
            planned_object_key: data_file.object_key_hint.clone(),
            table_name: data_file.table_name.clone(),
            relation: data_file.relation.clone(),
            source_bucket: data_file.source_bucket,
            epoch_id: write_plan.epoch_id.clone(),
            file_uri: file_uri.into(),
            file_format: "parquet".to_string(),
            file_size_in_bytes,
            content_sha256,
            object_version,
            record_count,
            checksum_rollup: data_file.checksum_rollup,
        },
    })
}

pub(super) fn parquet_error(
    operation: &'static str,
    error: parquet::errors::ParquetError,
) -> LakeError {
    LakeError::RawCdcParquet {
        operation,
        message: error.to_string(),
    }
}
