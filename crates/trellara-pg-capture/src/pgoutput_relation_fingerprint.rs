use trellara_protocol::ReplicaIdentity;
use xxhash_rust::xxh3::xxh3_64;

use crate::pgoutput_relation::PgOutputColumn;

pub(crate) fn schema_fingerprint(
    replica_identity: ReplicaIdentity,
    columns: &[PgOutputColumn],
) -> u64 {
    let mut input = String::new();
    input.push_str(&(replica_identity as i32).to_string());
    input.push('\x1e');
    for (index, column) in columns.iter().enumerate() {
        input.push_str(&(index + 1).to_string());
        input.push('\x1f');
        input.push_str(&column.name);
        input.push('\x1f');
        input.push_str(&column.type_oid.to_string());
        input.push('\x1f');
        input.push_str(&column.type_modifier.to_string());
        input.push('\x1f');
        input.push_str(if column.is_key { "key" } else { "value" });
        input.push('\x1e');
    }
    xxh3_64(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn relation_columns(type_modifier: i32) -> Vec<PgOutputColumn> {
        vec![PgOutputColumn {
            is_key: false,
            name: "name".to_string(),
            type_oid: 1043,
            type_modifier,
        }]
    }

    #[test]
    fn schema_fingerprint_includes_postgres_type_modifier() {
        let varchar_32 = relation_columns(36);
        let varchar_64 = relation_columns(68);

        assert_ne!(
            schema_fingerprint(ReplicaIdentity::Default, &varchar_32),
            schema_fingerprint(ReplicaIdentity::Default, &varchar_64)
        );
    }
}
