use std::collections::BTreeMap;

use crate::{LakeError, LakeRawCdcDataFilePlan, LakeRawCdcEpochWritePlan, LakeRawCdcRowIntent};

pub(super) fn rows_for_file<'a>(
    write_plan: &'a LakeRawCdcEpochWritePlan,
    data_file: &LakeRawCdcDataFilePlan,
) -> Result<Vec<&'a LakeRawCdcRowIntent>, LakeError> {
    let rows = write_plan
        .row_intents
        .iter()
        .filter(|row| {
            row.relation == data_file.relation && row.source_bucket == data_file.source_bucket
        })
        .collect::<Vec<_>>();
    compare_count("record_count", data_file.change_count, rows.len())?;
    validate_transaction_checksums(&rows, data_file)?;
    if let Some(row) = rows.iter().find(|row| row.epoch_id != write_plan.epoch_id) {
        return Err(LakeError::RawCdcParquetBoundaryMismatch {
            field: "epoch_id",
            expected: write_plan.epoch_id.clone(),
            actual: row.epoch_id.clone(),
        });
    }
    Ok(rows)
}

pub(super) fn rows_len_u64(value: usize) -> Result<u64, LakeError> {
    u64::try_from(value).map_err(|_| LakeError::RawCdcParquetCountOverflow {
        field: "record_count",
    })
}

fn validate_transaction_checksums(
    rows: &[&LakeRawCdcRowIntent],
    data_file: &LakeRawCdcDataFilePlan,
) -> Result<(), LakeError> {
    let mut transaction_checksums = BTreeMap::<&str, u64>::new();
    for row in rows {
        if let Some(previous) =
            transaction_checksums.insert(row.transaction_id.as_str(), row.envelope_checksum)
        {
            if previous != row.envelope_checksum {
                return Err(boundary_mismatch(
                    "transaction_checksum",
                    previous,
                    row.envelope_checksum,
                ));
            }
        }
    }
    let checksum_rollup = transaction_checksums
        .values()
        .fold(0u64, |rollup, checksum| rollup ^ checksum);
    if checksum_rollup != data_file.checksum_rollup {
        return Err(boundary_mismatch(
            "checksum_rollup",
            data_file.checksum_rollup,
            checksum_rollup,
        ));
    }
    Ok(())
}

fn compare_count(field: &'static str, expected: usize, actual: usize) -> Result<(), LakeError> {
    if expected == actual {
        Ok(())
    } else {
        Err(LakeError::RawCdcParquetBoundaryMismatch {
            field,
            expected: expected.to_string(),
            actual: actual.to_string(),
        })
    }
}

fn boundary_mismatch(field: &'static str, expected: u64, actual: u64) -> LakeError {
    LakeError::RawCdcParquetBoundaryMismatch {
        field,
        expected: expected.to_string(),
        actual: actual.to_string(),
    }
}
