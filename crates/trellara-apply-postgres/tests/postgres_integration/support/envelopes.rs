use trellara_protocol::{
    idempotency_key, AffectedTable, ChangeRecord, ColumnValue, ManifestBoundaryMode,
    ManifestPartition, Operation, RelationId, RelationSchemaVersion, ReplicaIdentity, RowImage,
    StrictEnvelope, TransactionEnvelope, TransactionManifest,
};

use super::{DATABASE_ID, DATASET_ID, DEFAULT_TEST_DATABASE_URL, SOURCE_ID, TABLE_NAME};

#[path = "envelopes/changes.rs"]
mod changes;
#[path = "envelopes/envelope.rs"]
mod envelope;
#[path = "envelopes/values.rs"]
mod values;

pub(crate) use changes::*;
pub(crate) use envelope::*;
pub(crate) use values::*;

#[test]
fn integration_envelope_includes_schema_version_evidence() {
    let custom_relation = RelationId::new(99, "public", "custom_sales");
    let envelope = envelope(
        "tx-schema-evidence",
        "0/16B6C50",
        "0/16B6D28",
        vec![insert_change_for_relation(
            "tx-schema-evidence",
            1,
            custom_relation.clone(),
            "sale-1",
            "1299",
        )],
    );

    assert_eq!(
        envelope.schema_versions,
        vec![RelationSchemaVersion {
            relation: Some(custom_relation),
            version: 12_345,
        }]
    );
    envelope.verify_checksum().expect("fresh envelope checksum");
}

#[test]
fn default_integration_database_url_is_documented() {
    assert_eq!(
        DEFAULT_TEST_DATABASE_URL,
        "postgresql://trellara:trellara@localhost:55433/trellara_target"
    );
}
