use trellara_protocol::{
    idempotency_key, AffectedTable, ChangeRecord, ColumnValue, ManifestBoundaryMode,
    ManifestPartition, Operation, RelationId, ReplicaIdentity, RowImage, StrictEnvelope,
    TransactionEnvelope, TransactionManifest,
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
fn default_integration_database_url_is_documented() {
    assert_eq!(
        DEFAULT_TEST_DATABASE_URL,
        "postgresql://trellara:trellara@localhost:55433/trellara_target"
    );
}
