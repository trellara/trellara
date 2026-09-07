use sha2::{Digest, Sha256};
use trellara_lake::LakeRawCdcEpochWritePlan;

use crate::{planner::TableSeed, IcebergCompletedDataFile};

pub(crate) fn epoch_commit_id<'a>(
    write_plan: &LakeRawCdcEpochWritePlan,
    tables: impl Iterator<Item = &'a TableSeed>,
) -> String {
    let mut hasher = Sha256::new();
    hash_component(&mut hasher, &write_plan.dataset_id);
    hash_component(&mut hasher, &write_plan.epoch_id);
    hash_component(
        &mut hasher,
        &write_plan.epoch_metadata.epoch_row.manifest_digest,
    );
    hash_component(&mut hasher, &write_plan.checksum_rollup.to_string());
    for table in tables {
        hash_component(&mut hasher, &table.lake_table_name);
        hash_component(&mut hasher, &table.target.qualified_name());
        let mut files = table.data_files.iter().collect::<Vec<_>>();
        files.sort_by(|left, right| left.planned_object_key.cmp(&right.planned_object_key));
        for file in files {
            hash_file(&mut hasher, file);
        }
    }
    hex_digest(&hasher.finalize())
}

pub(crate) fn table_commit_id(epoch_commit_id: &str, table: &TableSeed) -> String {
    let mut hasher = Sha256::new();
    hash_component(&mut hasher, epoch_commit_id);
    hash_component(&mut hasher, &table.lake_table_name);
    hash_component(&mut hasher, &table.target.qualified_name());
    for file in &table.data_files {
        hash_file(&mut hasher, file);
    }
    hex_digest(&hasher.finalize())
}

pub(crate) fn uuid_from_hex_digest(digest: &str) -> String {
    let mut bytes = [0u8; 16];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&digest[start..start + 2], 16)
            .expect("SHA-256 digest contains hexadecimal bytes");
    }
    bytes[6] = (bytes[6] & 0x0f) | 0x50;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15],
    )
}

fn hash_file(hasher: &mut Sha256, file: &IcebergCompletedDataFile) {
    hash_component(hasher, &file.planned_object_key);
    hash_component(hasher, &file.file_uri);
    hash_component(hasher, &file.file_size_in_bytes.to_string());
    hash_component(hasher, &file.record_count.to_string());
    hash_component(hasher, &file.checksum_rollup.to_string());
}

fn hash_component(hasher: &mut Sha256, component: &str) {
    hasher.update(component.len().to_string().as_bytes());
    hasher.update(b":");
    hasher.update(component.as_bytes());
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write;
        write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}
