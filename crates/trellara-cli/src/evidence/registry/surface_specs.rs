pub(crate) struct EvidenceRegistrySurfaceSpec {
    pub(crate) code: &'static str,
    pub(crate) artifact_name: &'static str,
    pub(crate) evidence: &'static str,
}

pub(crate) const EVIDENCE_REGISTRY_SURFACE_SPECS: &[EvidenceRegistrySurfaceSpec] = &[
    EvidenceRegistrySurfaceSpec {
        code: "source_safety",
        artifact_name: "source-safety-checklist.md",
        evidence: "read-only CDC source risk, WAL, slot, and identity review",
    },
    EvidenceRegistrySurfaceSpec {
        code: "transaction_boundary",
        artifact_name: "consistency-contract.json",
        evidence: "source acknowledgement, visibility, replay, and checkpoint contracts",
    },
    EvidenceRegistrySurfaceSpec {
        code: "performance_envelope",
        artifact_name: "performance-envelope.json",
        evidence: "bounded transaction and durability tradeoff review",
    },
    EvidenceRegistrySurfaceSpec {
        code: "identity_audit",
        artifact_name: "identity-audit.json",
        evidence: "ordinary primary-key/no-FULL and TOAST preservation evidence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "consumer_semantics",
        artifact_name: "consumer-semantics.json",
        evidence: "strict versus partitioned visibility language for downstream teams",
    },
    EvidenceRegistrySurfaceSpec {
        code: "partition_rebalance_plan",
        artifact_name: "partition-rebalance-plan.json",
        evidence: "partitioned scale rebalance governance with runtime ownership movement disabled until reviewed cutover evidence exists",
    },
    EvidenceRegistrySurfaceSpec {
        code: "fleet_readiness",
        artifact_name: "fleet-scorecard.txt",
        evidence: "design-partner fleet go/no-go scorecard",
    },
    EvidenceRegistrySurfaceSpec {
        code: "fleet_evidence_plan",
        artifact_name: "fleet-evidence-plan.txt",
        evidence: "live-evidence command sequence before fleet readiness is declared",
    },
    EvidenceRegistrySurfaceSpec {
        code: "live_evidence_template",
        artifact_name: "live-evidence/collect.sh",
        evidence: "reviewable live-evidence collection script with gate-specific artifact names",
    },
    EvidenceRegistrySurfaceSpec {
        code: "control_plane_pull",
        artifact_name: "fleet-control-plane.json",
        evidence: "hosted capability pull, deferral, and evidence gaps",
    },
    EvidenceRegistrySurfaceSpec {
        code: "support_diagnostics",
        artifact_name: "diagnostics.json",
        evidence: "support escalation, repair-plan, metrics, latest failure evidence, and stream inspect-local recovery readiness fields",
    },
    EvidenceRegistrySurfaceSpec {
        code: "single_review_bundle",
        artifact_name: "proof-bundle.md",
        evidence: "human-readable proof chain for executive and platform review",
    },
    EvidenceRegistrySurfaceSpec {
        code: "schema_ddl_plan",
        artifact_name: "schema-ddl-plan.json",
        evidence: "policy-gated DDL propagation, schema-barrier release gates, and post-DDL DML visibility evidence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "schema_ddl_apply_plan",
        artifact_name: "schema-ddl-apply-plan.json",
        evidence: "dry-run target Postgres DDL apply plan with safe SQL and barrier release sequence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "schema_ddl_envelope_plan",
        artifact_name: "schema-ddl-envelope-plan.json",
        evidence: "runtime DDL envelope transaction barrier, event classification, propagation boundary, propagation decisions, policy digest, and replay-envelope proof with ddl_events stripped",
    },
    EvidenceRegistrySurfaceSpec {
        code: "ddl_barrier_status",
        artifact_name: "ddl-barrier-status.json",
        evidence: "DDL barrier release gate with pending/rejected/unexpected ACK blockers and stable release_blocker_codes",
    },
    EvidenceRegistrySurfaceSpec {
        code: "ddl_release_proof",
        artifact_name: "ddl-release-proof.json",
        evidence: "DDL barrier release proof with cdc_transaction_boundary, required sink ACK evidence, and post-DDL DML visibility",
    },
    EvidenceRegistrySurfaceSpec {
        code: "lake_writer_plan",
        artifact_name: "lake-writer-plan.json",
        evidence: "raw CDC append-file and epoch-metadata writer intent with transaction ordering, DDL boundary metadata, durability gates, and duplicate replay accounting",
    },
    EvidenceRegistrySurfaceSpec {
        code: "sample_envelope",
        artifact_name: "sample-envelope.pb",
        evidence: "deterministic encoded transaction envelope used by the package lake writer-plan and fan-in run commands",
    },
    EvidenceRegistrySurfaceSpec {
        code: "lake_fanin_run",
        artifact_name: "lake-fanin-run.json",
        evidence: "bounded lake fan-in dry-run verdict with replay safety and Spark release gates",
    },
    EvidenceRegistrySurfaceSpec {
        code: "lake_completeness",
        artifact_name: "lake-completeness.json",
        evidence: "Iceberg completeness evidence tying epoch, verification, writer recovery, and Spark gates together",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_current_state_template",
        artifact_name: "spark-current-state.sql",
        evidence: "epoch-gated Spark current-state derivation with idempotent merge semantics",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_current_state_runner",
        artifact_name: "spark-current-state.py",
        evidence: "PySpark runner that refuses unresolved placeholders and prints template_sha256 for current-state DDL ACK evidence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_scd2_template",
        artifact_name: "spark-scd2.sql",
        evidence: "epoch-gated Spark SCD2 derivation with idempotent valid-time semantics",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_scd2_runner",
        artifact_name: "spark-scd2.py",
        evidence: "PySpark runner that refuses unresolved placeholders and prints template_sha256 for SCD2 DDL ACK evidence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_maintenance_template",
        artifact_name: "spark-maintenance.sql",
        evidence: "verification-gated Spark compaction and snapshot expiry schedule",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_maintenance_runner",
        artifact_name: "spark-maintenance.py",
        evidence: "PySpark runner that executes table maintenance only from rendered SQL and prints template_sha256 for DDL ACK evidence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_completeness_dashboard",
        artifact_name: "spark-completeness-dashboard.sql",
        evidence: "epoch-gated Spark completeness dashboard for source, gap, quarantine, and verification review",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_completeness_dashboard_runner",
        artifact_name: "spark-completeness-dashboard.py",
        evidence: "PySpark runner that refuses unresolved placeholders and prints template_sha256 for completeness dashboard DDL ACK evidence",
    },
    EvidenceRegistrySurfaceSpec {
        code: "spark_golden_fixture",
        artifact_name: "spark-golden-fixture.json",
        evidence: "deterministic raw CDC fixture with expected current-state and SCD2 golden outputs",
    },
];
