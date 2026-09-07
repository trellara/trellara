use crate::source_inspection::{replica_identity_contract_notes, schema_fingerprint};
use trellara_protocol::ReplicaIdentity;

use super::*;

#[test]
fn schema_fingerprint_is_stable_and_sensitive() {
    let columns = keyed_amount_columns();
    let mut changed = columns.clone();
    changed[1].nullable = true;

    assert_eq!(schema_fingerprint(&columns), schema_fingerprint(&columns));
    assert_ne!(schema_fingerprint(&columns), schema_fingerprint(&changed));
}

#[test]
fn default_replica_identity_with_primary_key_is_contract_note_not_issue() {
    let columns = vec![
        TableColumn {
            ordinal_position: 1,
            name: "id".to_string(),
            type_oid: 25,
            type_name: "text".to_string(),
            nullable: false,
            is_key: true,
        },
        TableColumn {
            ordinal_position: 2,
            name: "description".to_string(),
            type_oid: 25,
            type_name: "text".to_string(),
            nullable: true,
            is_key: false,
        },
    ];
    let notes = replica_identity_contract_notes(
        Some(ReplicaIdentity::Default),
        &["id".to_string()],
        &columns,
    );

    assert_eq!(notes.len(), 2);
    assert!(notes[0].contains("replica identity DEFAULT is accepted"));
    assert!(notes[0].contains("REPLICA IDENTITY FULL is not required"));
    assert!(notes[1].contains("unchanged TOAST values"));
    assert!(notes[1].contains("description"));
}

#[test]
fn full_replica_identity_reports_wal_tradeoff() {
    let notes = replica_identity_contract_notes(Some(ReplicaIdentity::Full), &[], &[]);

    assert_eq!(
        notes,
        vec![
            "replica identity FULL is safe but can increase WAL volume; ordinary primary-key tables can usually use DEFAULT".to_string()
        ]
    );
}

fn keyed_amount_columns() -> Vec<TableColumn> {
    vec![
        TableColumn {
            ordinal_position: 1,
            name: "id".to_string(),
            type_oid: 25,
            type_name: "text".to_string(),
            nullable: false,
            is_key: true,
        },
        TableColumn {
            ordinal_position: 2,
            name: "amount_cents".to_string(),
            type_oid: 20,
            type_name: "bigint".to_string(),
            nullable: false,
            is_key: false,
        },
    ]
}
