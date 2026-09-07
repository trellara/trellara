use crate::ChaosEnterpriseReviewGate;

pub(crate) fn default_enterprise_review_gates() -> Vec<ChaosEnterpriseReviewGate> {
    vec![
        ChaosEnterpriseReviewGate::new(
            "source_safety",
            "Is the source safe to capture?",
            "trellara check --config <flow> --format text",
            "No critical slot, WAL retention, replica identity, failover-slot, or subscription conflict blockers",
        ),
        ChaosEnterpriseReviewGate::new(
            "transaction_boundary",
            "Does CDC preserve transaction boundaries?",
            "trellara inspect-transaction --file <envelope.pb> --format text",
            "Checksum status is match, affected tables are explicit, manifest boundary_mode names strict chunk or partition semantics, and manifest counts bind to the commit marker",
        ),
        ChaosEnterpriseReviewGate::new(
            "snapshot_handoff",
            "Can a first snapshot hand off to the stream safely?",
            "trellara snapshot --config <flow> --run-id <run>",
            "Every selected table reaches stream_handoff_ready with a durable handoff watermark",
        ),
        ChaosEnterpriseReviewGate::new(
            "target_convergence",
            "Has the target converged?",
            "trellara verify --config <flow>",
            "Row counts and checksums match at a caught-up target checkpoint",
        ),
        ChaosEnterpriseReviewGate::new(
            "recoverability",
            "Can operators recover without silent skips?",
            "trellara status --config <flow> --view diagnostics --format text",
            "Quarantine, repair-plan, metrics, and replay-ready commands identify the safe next action",
        ),
        ChaosEnterpriseReviewGate::new(
            "lane_d_qualification",
            "Have durable-boundary failure harnesses and observability assertions passed?",
            "trellara chaos report --output docs/correctness-report.html",
            "Source promotion, broker quorum loss, target restart, object-store/catalog split, and 24-hour large-transaction soak all pass with named recovery and metric evidence",
        ),
        ChaosEnterpriseReviewGate::new(
            "partitioned_visibility",
            "Can partitioned scale expose global visibility safely?",
            "trellara partition-watermarks --config <flow>",
            "Every partition has checkpoint evidence before global current-state visibility advances",
        ),
        ChaosEnterpriseReviewGate::new(
            "auditable_package",
            "Can the proof package be shared and audited?",
            "trellara pilot-package --config <flow>",
            "proof-bundle.md and manifest.json bind the review chain to SHA-256 digests",
        ),
    ]
}

impl ChaosEnterpriseReviewGate {
    fn new(
        code: impl Into<String>,
        question: impl Into<String>,
        proof_surface: impl Into<String>,
        gate: impl Into<String>,
    ) -> Self {
        Self {
            code: code.into(),
            question: question.into(),
            proof_surface: proof_surface.into(),
            gate: gate.into(),
        }
    }
}
