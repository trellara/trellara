use crate::{
    ManifestBoundaryMode, PartitionedScaleDecision, TransactionBoundaryKind, TransactionEnvelope,
    TransactionManifest,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionedScaleManifestEvidence {
    pub boundary_mode: ManifestBoundaryMode,
    pub manifest_checksum: u64,
    pub global_event_count: u32,
    pub manifest_event_count: u32,
    pub envelope_event_count: usize,
    pub event_count_coverage: bool,
    pub participating_partition_count: usize,
    pub participating_partition_ids: Vec<u32>,
    pub source_commit_lsn: String,
    pub source_commit_timestamp_ms: i64,
    pub visibility_contract: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PartitionedScaleReadiness {
    pub transaction_id: String,
    pub boundary_kind: TransactionBoundaryKind,
    pub decision: PartitionedScaleDecision,
    pub partition_parallel_safe: bool,
    pub requires_ddl_barrier: bool,
    pub dml_replay_after_ddl_barrier_required: bool,
    pub manifest_evidence: Option<PartitionedScaleManifestEvidence>,
    pub reason: &'static str,
}

impl TransactionEnvelope {
    pub fn partitioned_scale_readiness(&self) -> PartitionedScaleReadiness {
        let boundary_kind = self.boundary_kind();
        let decision = self.partitioned_scale_decision();
        let requires_ddl_barrier = self.requires_ddl_barrier();
        PartitionedScaleReadiness {
            transaction_id: self.transaction_id.clone(),
            boundary_kind,
            decision,
            partition_parallel_safe: decision == PartitionedScaleDecision::PartitionParallelDml,
            requires_ddl_barrier,
            dml_replay_after_ddl_barrier_required: requires_ddl_barrier && !self.changes.is_empty(),
            manifest_evidence: self
                .manifest
                .as_ref()
                .map(|manifest| PartitionedScaleManifestEvidence::from_manifest(manifest, self)),
            reason: partitioned_scale_readiness_reason(boundary_kind),
        }
    }
}

impl PartitionedScaleManifestEvidence {
    pub fn from_manifest(manifest: &TransactionManifest, envelope: &TransactionEnvelope) -> Self {
        Self::from_manifest_counts(manifest, envelope.changes.len())
    }

    pub fn from_manifest_counts(
        manifest: &TransactionManifest,
        envelope_event_count: usize,
    ) -> Self {
        let manifest_event_count = manifest
            .partitions
            .iter()
            .map(|partition| partition.event_count)
            .sum();
        Self {
            boundary_mode: ManifestBoundaryMode::try_from(manifest.boundary_mode)
                .unwrap_or(ManifestBoundaryMode::Unspecified),
            manifest_checksum: manifest.compute_checksum(),
            global_event_count: manifest.global_event_count,
            manifest_event_count,
            envelope_event_count,
            event_count_coverage: manifest.global_event_count as usize == envelope_event_count
                && manifest_event_count as usize == envelope_event_count,
            participating_partition_count: manifest.partitions.len(),
            participating_partition_ids: manifest
                .partitions
                .iter()
                .map(|partition| partition.id)
                .collect(),
            source_commit_lsn: manifest.source_commit_lsn.clone(),
            source_commit_timestamp_ms: manifest.source_commit_timestamp_ms,
            visibility_contract: manifest_visibility_contract(manifest),
        }
    }
}

fn partitioned_scale_readiness_reason(boundary_kind: TransactionBoundaryKind) -> &'static str {
    match boundary_kind {
        TransactionBoundaryKind::Empty => "empty transaction has no DML to partition",
        TransactionBoundaryKind::DmlOnly => {
            "DML-only transaction can be partitioned without a DDL barrier"
        }
        TransactionBoundaryKind::DdlOnly => "DDL-only transaction must use the DDL barrier path",
        TransactionBoundaryKind::MixedDdlAndDml => {
            "mixed DDL and DML transaction must apply DDL barrier before partitioned DML replay"
        }
    }
}

fn manifest_visibility_contract(manifest: &TransactionManifest) -> &'static str {
    match ManifestBoundaryMode::try_from(manifest.boundary_mode)
        .unwrap_or(ManifestBoundaryMode::Unspecified)
    {
        ManifestBoundaryMode::PartitionedScale => {
            "global visibility waits for manifest, commit marker, and every participating partition"
        }
        ManifestBoundaryMode::StrictChunkedTransactionOrder => {
            "global visibility waits for manifest, commit marker, and every strict-order chunk"
        }
        ManifestBoundaryMode::Unspecified => {
            "global visibility waits for manifest and all declared transaction fragments"
        }
    }
}
