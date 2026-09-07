use ::parquet::arrow::arrow_reader::ParquetRecordBatchReader;
use std::collections::BTreeMap;

use arrow_array::{Int32Array, StringArray};
use bytes::Bytes;

use super::*;

#[test]
fn raw_cdc_parquet_writer_round_trips_rows_and_field_ids() {
    let first = envelope_for(
        "store-001",
        "tx-1",
        "0/16B6C50",
        vec![
            change(
                Operation::Insert,
                1,
                None,
                Some(row("sale-1", "10", "2026-01-01")),
            ),
            change(
                Operation::Update,
                2,
                Some(row("sale-1", "10", "2026-01-01")),
                Some(row("sale-1", "20", "2026-01-02")),
            ),
        ],
    );
    let plan = plan_raw_cdc_epoch_writes(
        &raw_writer_config(1),
        &config(),
        std::slice::from_ref(&first),
    )
    .expect("raw CDC writer plan");
    let data_file = &plan.data_files[0];

    let output = write_raw_cdc_parquet_file(
        Vec::new(),
        &plan,
        data_file,
        "s3://lake/epoch-2026-08-16T06/sales/source_bucket=0/data.parquet",
        Some("version-1".to_string()),
    )
    .expect("write parquet");

    assert_eq!(
        output.evidence.planned_object_key,
        data_file.object_key_hint
    );
    assert_eq!(
        output.evidence.file_size_in_bytes as usize,
        output.writer.len()
    );
    assert_eq!(output.evidence.record_count, 2);
    assert_eq!(output.evidence.checksum_rollup, first.checksum);
    assert_eq!(
        output.evidence.object_version,
        Some("version-1".to_string())
    );
    assert!(output.evidence.content_sha256.len() == 64);

    let schema = raw_cdc_parquet_arrow_schema();
    assert_eq!(
        schema.field(0).metadata().get(RAW_CDC_PARQUET_FIELD_ID_KEY),
        Some(&"1".to_string())
    );
    assert_eq!(
        schema
            .field(25)
            .metadata()
            .get(RAW_CDC_PARQUET_FIELD_ID_KEY),
        Some(&"26".to_string())
    );

    let mut reader = ParquetRecordBatchReader::try_new(Bytes::from(output.writer), 1024)
        .expect("open parquet reader");
    let batch = reader.next().expect("record batch").expect("read batch");
    assert_eq!(batch.num_rows(), 2);

    let relation = batch
        .column(4)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("relation column");
    assert_eq!(relation.value(0), "public.sales");
    let operation = batch
        .column(10)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("operation column");
    assert_eq!(operation.value(1), "update");
    let source_bucket = batch
        .column(1)
        .as_any()
        .downcast_ref::<Int32Array>()
        .expect("source bucket column");
    assert_eq!(source_bucket.value(0), 0);
    let epoch = batch
        .column(25)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("epoch column");
    assert_eq!(epoch.value(0), "epoch-2026-08-16T06");
}

#[test]
fn raw_cdc_parquet_writer_rejects_plan_mismatches() {
    let first = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    let mut plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config(), &[first])
        .expect("raw CDC writer plan");
    plan.data_files[0].change_count = 2;

    assert!(matches!(
        write_raw_cdc_parquet_file(
            Vec::new(),
            &plan,
            &plan.data_files[0],
            "s3://lake/data.parquet",
            None
        ),
        Err(LakeError::RawCdcParquetBoundaryMismatch {
            field: "record_count",
            ..
        })
    ));
}

#[test]
fn raw_cdc_epoch_metadata_parquet_writer_round_trips_completeness_row() {
    let first = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config(), &[first])
        .expect("raw CDC writer plan");
    let snapshots = BTreeMap::from([("analytics.retail.sales_cdc".to_string(), 101)]);

    let output = write_raw_cdc_epoch_metadata_parquet_file(
        Vec::new(),
        &plan,
        "trellara-iceberg-epoch:commit-1",
        snapshots.clone(),
        "s3://lake/retail/_trellara_epochs/epoch_id=epoch-2026-08-16T06/part-00000.parquet",
        Some("metadata-version-1".to_string()),
    )
    .expect("write epoch metadata parquet");

    assert_eq!(
        output.evidence.planned_object_key,
        "retail__trellara__fanin___trellara_epochs/epoch_id=epoch-2026-08-16T06/part-00000.parquet"
    );
    assert_eq!(output.evidence.table_name, plan.epoch_metadata.epochs_table);
    assert_eq!(
        output.evidence.iceberg_snapshot_id,
        "trellara-iceberg-epoch:commit-1"
    );
    assert_eq!(output.evidence.raw_table_snapshot_ids, snapshots);
    assert_eq!(output.evidence.record_count, 1);
    assert_eq!(
        output.evidence.file_size_in_bytes as usize,
        output.writer.len()
    );

    let schema = raw_cdc_epoch_metadata_parquet_arrow_schema();
    assert_eq!(
        schema.field(0).metadata().get(RAW_CDC_PARQUET_FIELD_ID_KEY),
        Some(&"1".to_string())
    );
    assert_eq!(
        schema
            .field(15)
            .metadata()
            .get(RAW_CDC_PARQUET_FIELD_ID_KEY),
        Some(&"16".to_string())
    );

    let mut reader = ParquetRecordBatchReader::try_new(Bytes::from(output.writer), 1024)
        .expect("open epoch metadata parquet reader");
    let batch = reader.next().expect("record batch").expect("read batch");
    assert_eq!(batch.num_rows(), 1);
    let state = batch
        .column(2)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("state column");
    assert_eq!(state.value(0), "complete");
    let snapshot_ref = batch
        .column(14)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("snapshot ref column");
    assert_eq!(snapshot_ref.value(0), "trellara-iceberg-epoch:commit-1");
    let raw_snapshots = batch
        .column(15)
        .as_any()
        .downcast_ref::<StringArray>()
        .expect("snapshot map column");
    assert_eq!(
        raw_snapshots.value(0),
        "{\"analytics.retail.sales_cdc\":101}"
    );
}

#[test]
fn raw_cdc_epoch_metadata_parquet_writer_requires_raw_snapshot_evidence() {
    let plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config(), &[])
        .expect("raw CDC writer plan");

    assert!(matches!(
        write_raw_cdc_epoch_metadata_parquet_file(
            Vec::new(),
            &plan,
            "trellara-iceberg-epoch:commit-1",
            BTreeMap::new(),
            "s3://lake/metadata.parquet",
            None,
        ),
        Err(LakeError::InvalidRawCdcEpochMetadataField {
            field: "epoch.raw_table_snapshot_ids",
            ..
        })
    ));
}

#[test]
fn raw_cdc_epoch_metadata_parquet_writer_rejects_non_consumable_epoch() {
    let first = envelope(vec![change(
        Operation::Insert,
        1,
        None,
        Some(row("sale-1", "10", "2026-01-01")),
    )]);
    let mut plan = plan_raw_cdc_epoch_writes(&raw_writer_config(1), &config(), &[first])
        .expect("raw CDC writer plan");
    plan.epoch_metadata.epoch_row.state = LakeCompletenessState::Open;

    assert!(matches!(
        write_raw_cdc_epoch_metadata_parquet_file(
            Vec::new(),
            &plan,
            "trellara-iceberg-epoch:commit-1",
            BTreeMap::from([("analytics.retail.sales_cdc".to_string(), 101)]),
            "s3://lake/metadata.parquet",
            None,
        ),
        Err(LakeError::EpochCompletenessStateMismatch { .. })
            | Err(LakeError::EpochNotConsumable { .. })
    ));
}
