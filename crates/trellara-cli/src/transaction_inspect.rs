use serde::Serialize;
use trellara_protocol::TransactionEnvelope;

use crate::{
    transaction_inspect_ddl_events, transaction_inspect_dml_replay, transaction_inspect_tables,
    ChecksumStatus, TransactionBoundaryStatus, TransactionInspectDdlEvent,
    TransactionInspectDmlReplay,
};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectSummary {
    pub(crate) protocol_version: u32,
    pub(crate) source_id: String,
    pub(crate) database_id: String,
    pub(crate) dataset_id: String,
    pub(crate) transaction_id: String,
    pub(crate) begin_lsn: String,
    pub(crate) commit_lsn: String,
    pub(crate) commit_timestamp_ms: i64,
    pub(crate) checksum: u64,
    pub(crate) event_count: usize,
    pub(crate) ddl_event_count: usize,
    pub(crate) source_event_count: usize,
    pub(crate) schema_version_count: usize,
    pub(crate) ddl_events: Vec<TransactionInspectDdlEvent>,
    pub(crate) dml_replay_after_ddl_barrier: Option<TransactionInspectDmlReplay>,
    pub(crate) transaction_boundary: TransactionInspectBoundaryProof,
    pub(crate) affected_tables: Vec<TransactionInspectTable>,
    pub(crate) partition_manifest: Option<TransactionInspectManifest>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectTable {
    pub(crate) relation: String,
    pub(crate) event_count: usize,
    pub(crate) inserts: usize,
    pub(crate) updates: usize,
    pub(crate) deletes: usize,
    pub(crate) truncates: usize,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectManifest {
    pub(crate) boundary_mode: String,
    pub(crate) global_event_count: u32,
    pub(crate) participating_partition_count: usize,
    pub(crate) checksum: u64,
    pub(crate) source_commit_lsn: String,
    pub(crate) source_commit_timestamp_ms: i64,
    pub(crate) partitions: Vec<TransactionInspectPartition>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectPartition {
    pub(crate) id: u32,
    pub(crate) event_count: u32,
    pub(crate) first_total_order: u32,
    pub(crate) last_total_order: u32,
    pub(crate) checksum: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct TransactionInspectBoundaryProof {
    pub(crate) mode: String,
    pub(crate) source_boundary_kind: String,
    pub(crate) partitioned_scale_decision: String,
    pub(crate) partition_parallel_safe: bool,
    pub(crate) requires_ddl_barrier: bool,
    pub(crate) dml_replay_after_ddl_barrier_required: bool,
    pub(crate) partitioned_scale_reason: String,
    pub(crate) status: TransactionBoundaryStatus,
    pub(crate) guarantee: String,
    pub(crate) visibility_contract: String,
    pub(crate) checksum_status: ChecksumStatus,
    pub(crate) manifest_barrier_required: bool,
    pub(crate) manifest_valid: bool,
    pub(crate) manifest_validation_error: Option<String>,
    pub(crate) global_event_count_matches: Option<bool>,
    pub(crate) partition_event_count_matches: Option<bool>,
    pub(crate) partitioned_scale_manifest_checksum: Option<u64>,
    pub(crate) partitioned_scale_manifest_event_count: Option<u32>,
    pub(crate) partitioned_scale_envelope_event_count: Option<usize>,
    pub(crate) partitioned_scale_event_count_coverage: Option<bool>,
    pub(crate) partitioned_scale_participating_partition_ids: Vec<u32>,
    pub(crate) partitioned_scale_visibility_contract: Option<String>,
    pub(crate) participating_partition_count: usize,
    pub(crate) expected_commit_marker_manifest_checksum: Option<u64>,
}

impl TransactionInspectSummary {
    pub(crate) fn from_envelope(envelope: &TransactionEnvelope) -> Self {
        Self {
            protocol_version: envelope.protocol_version,
            source_id: envelope.source_id.clone(),
            database_id: envelope.database_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            transaction_id: envelope.transaction_id.clone(),
            begin_lsn: envelope.begin_lsn.clone(),
            commit_lsn: envelope.commit_lsn.clone(),
            commit_timestamp_ms: envelope.commit_timestamp_ms,
            checksum: envelope.checksum,
            event_count: envelope.changes.len(),
            ddl_event_count: envelope.ddl_events.len(),
            source_event_count: envelope.changes.len() + envelope.ddl_events.len(),
            schema_version_count: envelope.schema_versions.len(),
            ddl_events: transaction_inspect_ddl_events(envelope),
            dml_replay_after_ddl_barrier: transaction_inspect_dml_replay(envelope),
            transaction_boundary: TransactionInspectBoundaryProof::from_envelope(envelope),
            affected_tables: transaction_inspect_tables(&envelope.changes),
            partition_manifest: envelope
                .manifest
                .as_ref()
                .map(TransactionInspectManifest::from_manifest),
        }
    }
}
