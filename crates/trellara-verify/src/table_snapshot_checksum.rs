use xxhash_rust::xxh3::xxh3_64;

use crate::TableSnapshot;

pub(crate) fn table_snapshot_checksum(snapshot: &TableSnapshot) -> u64 {
    let canonical = snapshot.clone().canonical();
    let mut bytes = Vec::new();
    bytes.extend_from_slice(canonical.relation.display_name().as_bytes());
    bytes.push(0);
    for row in canonical.rows {
        bytes.extend_from_slice(row.primary_key.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(row.row_hash.to_string().as_bytes());
        bytes.push(0);
    }
    xxh3_64(&bytes)
}
