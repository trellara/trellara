use super::*;

pub(super) fn assert_evidence_brief(contents: &PilotPackageContents) {
    assert_content_contains(
        "executive evidence",
        &contents.executive_evidence,
        &[
            "Trellara Executive Evidence Brief",
            "transaction-boundary proof",
            "quarantine reason",
            "brokerless local stream",
        ],
    );
    assert_content_contains(
        "enterprise evaluation",
        &contents.enterprise_evaluation,
        &[
            "Trellara enterprise evaluation",
            "recommended_mode: strict_transaction_order",
            "manifest boundary_mode",
        ],
    );
    assert_content_contains(
        "enterprise evaluation json",
        &contents.enterprise_evaluation_json,
        &["\"recommended_mode\": \"strict_transaction_order\""],
    );
    assert_content_contains(
        "proof bundle",
        &contents.proof_bundle,
        &[
            "Trellara Proof Bundle",
            "Enterprise evaluation",
            "trellara evaluate --config",
            "inspect-transaction --file <envelope.pb> --format text",
            "manifest `boundary_mode`",
            "snapshot_handoff_blocker_codes",
            "snapshot_handoff_recovery_actions",
            "strict chunk or partition manifest event-count checks",
            "Boundary Contract",
            "Brokerless stream inspection",
            "runtime barrier_pending_blockers",
            "Package Integrity",
        ],
    );
}
