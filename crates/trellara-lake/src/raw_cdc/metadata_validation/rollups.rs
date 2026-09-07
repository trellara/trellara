use crate::raw_cdc::metadata::RawCdcMetadataInput;
use crate::LakeError;

pub(super) fn validate(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    validate_source_rollups(input)?;
    validate_table_rollups(input)?;
    validate_partition_rollups(input)
}

fn validate_source_rollups(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    if input.source_rows.is_empty() {
        return Ok(());
    }

    let transaction_count = input.source_rows.iter().try_fold(0usize, |sum, source| {
        sum.checked_add(source.transaction_count)
            .ok_or(LakeError::RawCdcCountOverflow {
                field: "source_transaction_count",
            })
    })?;
    validate_source_rollup(
        input,
        "transaction_count",
        input.transaction_count as u128,
        transaction_count as u128,
    )?;

    let change_count = input.source_rows.iter().try_fold(0usize, |sum, source| {
        sum.checked_add(source.change_count)
            .ok_or(LakeError::RawCdcCountOverflow {
                field: "source_change_count",
            })
    })?;
    validate_source_rollup(
        input,
        "change_count",
        input.change_count as u128,
        change_count as u128,
    )?;

    let checksum_rollup = input
        .source_rows
        .iter()
        .fold(0u64, |rollup, source| rollup ^ source.checksum_rollup);
    validate_source_rollup(
        input,
        "checksum_rollup",
        input.checksum_rollup as u128,
        checksum_rollup as u128,
    )
}

fn validate_table_rollups(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    if input.table_rows.is_empty() {
        return Ok(());
    }

    let change_count = input.table_rows.iter().try_fold(0usize, |sum, table| {
        sum.checked_add(table.change_count)
            .ok_or(LakeError::RawCdcCountOverflow {
                field: "table_change_count",
            })
    })?;
    if change_count == input.change_count {
        Ok(())
    } else {
        Err(LakeError::RawCdcEpochTableRollupMismatch {
            epoch_id: input.epoch_id.clone(),
            field: "change_count",
            expected: input.change_count as u128,
            actual: change_count as u128,
        })
    }
}

fn validate_partition_rollups(input: &RawCdcMetadataInput) -> Result<(), LakeError> {
    if input.partition_rows.is_empty() {
        return Ok(());
    }

    let event_count = input
        .partition_rows
        .iter()
        .try_fold(0usize, |sum, partition| {
            sum.checked_add(partition.event_count)
                .ok_or(LakeError::RawCdcCountOverflow {
                    field: "partition_event_count",
                })
        })?;
    if event_count == input.change_count {
        Ok(())
    } else {
        Err(LakeError::RawCdcEpochPartitionRollupMismatch {
            epoch_id: input.epoch_id.clone(),
            field: "event_count",
            expected: input.change_count as u128,
            actual: event_count as u128,
        })
    }
}

fn validate_source_rollup(
    input: &RawCdcMetadataInput,
    field: &'static str,
    expected: u128,
    actual: u128,
) -> Result<(), LakeError> {
    if expected == actual {
        Ok(())
    } else {
        Err(LakeError::RawCdcEpochSourceRollupMismatch {
            epoch_id: input.epoch_id.clone(),
            field,
            expected,
            actual,
        })
    }
}
