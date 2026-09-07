use super::*;
use crate::barrier::{HeaderContext, PendingChunk, PendingCommitMarker, PendingManifest};
use crate::barrier_manifest_evidence_headers::ManifestEvidenceHeaders;
use trellara_protocol::{
    AffectedTable, ManifestBoundaryMode, ManifestPartition, PartitionChunk, RelationId,
    TransactionCommitMarker, TransactionManifest,
};

#[test]
fn pending_stats_saturating_add_prevents_wraparound() {
    let pending = PendingBarrierTransaction::default();
    let mut stats = BarrierPendingStats {
        transactions: u64::MAX,
        missing_manifest: u64::MAX,
        missing_commit_marker: u64::MAX,
        ..Default::default()
    };

    stats.add_pending(&pending);

    assert_eq!(stats.transactions, u64::MAX);
    assert_eq!(stats.missing_manifest, u64::MAX);
    assert_eq!(stats.missing_commit_marker, u64::MAX);
}

#[test]
fn pending_stats_reports_invalid_commit_marker_separately() {
    let manifest = manifest();
    let mut marker = TransactionCommitMarker::from_manifest(&manifest).expect("commit marker");
    marker.global_event_count += 1;
    let pending = PendingBarrierTransaction {
        manifest: Some(PendingManifest {
            manifest,
            context: header_context(),
            messages: Vec::new(),
        }),
        commit_marker: Some(PendingCommitMarker {
            marker,
            messages: Vec::new(),
        }),
        chunks: Default::default(),
    };

    let stats = BarrierPendingStats::from_pending([&pending]);

    assert_eq!(stats.transactions, 1);
    assert_eq!(stats.with_manifest, 1);
    assert_eq!(stats.with_commit_marker, 1);
    assert_eq!(stats.invalid_commit_marker, 1);
    assert_eq!(stats.missing_commit_marker, 0);
}

#[test]
fn pending_stats_reports_extra_chunks_separately() {
    let pending = PendingBarrierTransaction {
        manifest: Some(PendingManifest {
            manifest: manifest(),
            context: header_context(),
            messages: Vec::new(),
        }),
        commit_marker: None,
        chunks: [(0, chunk(0)), (99, chunk(99))].into_iter().collect(),
    };

    let stats = BarrierPendingStats::from_pending([&pending]);

    assert_eq!(stats.expected_chunks, 1);
    assert_eq!(stats.buffered_chunks, 2);
    assert_eq!(stats.missing_chunks, 0);
    assert_eq!(stats.extra_chunks, 1);
    assert_eq!(stats.complete_chunk_sets, 0);
}

fn manifest() -> TransactionManifest {
    TransactionManifest {
        transaction_id: "tx-marker".to_string(),
        source_commit_lsn: "0/16B7190".to_string(),
        source_commit_timestamp_ms: 1_786_420_000_000,
        global_event_count: 1,
        partitions: vec![ManifestPartition {
            id: 0,
            event_count: 1,
            first_total_order: 1,
            last_total_order: 1,
            checksum: 42,
        }],
        affected_tables: vec![AffectedTable {
            relation: Some(RelationId::new(42, "public", "sales")),
            event_count: 1,
        }],
        boundary_mode: ManifestBoundaryMode::PartitionedScale as i32,
    }
}

fn header_context() -> HeaderContext {
    HeaderContext {
        source_id: "source".to_string(),
        dataset_id: "sales".to_string(),
        database_id: "retail".to_string(),
        transaction_id: "tx-marker".to_string(),
        commit_lsn: "0/16B7190".to_string(),
        partitioned_scale_decision: "partition_parallel_dml".to_string(),
        partition_parallel_safe: "true".to_string(),
        requires_ddl_barrier: "false".to_string(),
        dml_replay_after_ddl_barrier_required: "false".to_string(),
        partitioned_scale_reason: "DML-only transaction can be partitioned without a DDL barrier"
            .to_string(),
        partition_id: String::new(),
        partition_event_count: String::new(),
        partition_checksum: String::new(),
        global_event_count: String::new(),
        partition_count: String::new(),
        manifest_checksum: String::new(),
        manifest_evidence: ManifestEvidenceHeaders::default(),
    }
}

fn chunk(partition_id: u32) -> PendingChunk {
    PendingChunk {
        chunk: PartitionChunk {
            transaction_id: "tx-marker".to_string(),
            partition_id,
            changes: Vec::new(),
            checksum: 0,
        },
        messages: Vec::new(),
    }
}
