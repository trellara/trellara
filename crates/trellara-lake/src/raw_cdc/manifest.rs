use crate::raw_cdc_types::{
    LakeRawCdcEpochPartitionRow, LakeRawCdcEpochSourceRow, LakeRawCdcEpochTableRow,
};
use sha2::{Digest, Sha256};

pub(crate) fn verification_id(dataset_id: &str, epoch_id: &str, checksum_rollup: u64) -> String {
    format!("verify:{dataset_id}:{epoch_id}:{checksum_rollup:016x}")
}

pub(super) fn epoch_manifest_digest(
    source_rows: &[LakeRawCdcEpochSourceRow],
    table_rows: &[LakeRawCdcEpochTableRow],
    partition_rows: &[LakeRawCdcEpochPartitionRow],
) -> String {
    let mut manifest = String::new();
    manifest.push_str("trellara_raw_cdc_epoch_manifest_v1\n");
    for source in source_rows {
        manifest.push_str(&format!(
            "source={}|{}|{}|{}|{}|{}|{}\n",
            source.epoch_id,
            source.source_id,
            source.start_lsn,
            source.end_lsn,
            source.transaction_count,
            source.change_count,
            source.checksum_rollup
        ));
    }
    for table in table_rows {
        manifest.push_str(&format!(
            "table={}|{}|{}|{}|{}\n",
            table.epoch_id,
            table.relation,
            table.transaction_count,
            table.change_count,
            table.checksum_rollup
        ));
    }
    for partition in partition_rows {
        manifest.push_str(&format!(
            "partition={}|{}|{}|{}|{}|{}|{}|{}\n",
            partition.epoch_id,
            partition.source_id,
            partition.partition_id,
            partition.first_commit_lsn,
            partition.last_commit_lsn,
            partition.transaction_count,
            partition.event_count,
            partition.checksum_rollup
        ));
    }
    format!("{:x}", Sha256::digest(manifest.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verification_id_encodes_checksum_as_fixed_width_hex() {
        assert_eq!(
            verification_id("retail", "epoch-1", 99),
            "verify:retail:epoch-1:0000000000000063"
        );
    }

    #[test]
    fn epoch_manifest_digest_changes_when_table_rollup_changes() {
        let source_rows = vec![source_row()];
        let mut left_tables = vec![table_row(7)];
        let right_tables = vec![table_row(8)];
        let partition_rows = vec![partition_row(9)];

        let left = epoch_manifest_digest(&source_rows, &left_tables, &partition_rows);
        let right = epoch_manifest_digest(&source_rows, &right_tables, &partition_rows);
        left_tables.reverse();

        assert_eq!(left.len(), 64);
        assert_ne!(left, right);
        assert_eq!(
            left,
            epoch_manifest_digest(&source_rows, &left_tables, &partition_rows)
        );
    }

    #[test]
    fn epoch_manifest_digest_changes_when_partition_rollup_changes() {
        let source_rows = vec![source_row()];
        let table_rows = vec![table_row(7)];
        let left_partitions = vec![partition_row(9)];
        let right_partitions = vec![partition_row(10)];

        let left = epoch_manifest_digest(&source_rows, &table_rows, &left_partitions);
        let right = epoch_manifest_digest(&source_rows, &table_rows, &right_partitions);

        assert_eq!(left.len(), 64);
        assert_ne!(left, right);
    }

    fn source_row() -> LakeRawCdcEpochSourceRow {
        LakeRawCdcEpochSourceRow {
            epoch_id: "epoch-1".to_string(),
            source_id: "source-a".to_string(),
            state: crate::LakeEpochSourceState::Complete,
            start_lsn: "0/16B0000".to_string(),
            end_lsn: "0/16B0100".to_string(),
            transaction_count: 1,
            change_count: 2,
            checksum_rollup: 3,
            lag_reason: None,
        }
    }

    fn table_row(checksum_rollup: u64) -> LakeRawCdcEpochTableRow {
        LakeRawCdcEpochTableRow {
            epoch_id: "epoch-1".to_string(),
            relation: "public.sales".to_string(),
            transaction_count: 1,
            change_count: 2,
            checksum_rollup,
        }
    }

    fn partition_row(checksum_rollup: u64) -> LakeRawCdcEpochPartitionRow {
        LakeRawCdcEpochPartitionRow {
            epoch_id: "epoch-1".to_string(),
            source_id: "source-a".to_string(),
            partition_id: 0,
            first_commit_lsn: "0/16B0000".to_string(),
            last_commit_lsn: "0/16B0100".to_string(),
            transaction_count: 1,
            event_count: 2,
            checksum_rollup,
        }
    }
}
