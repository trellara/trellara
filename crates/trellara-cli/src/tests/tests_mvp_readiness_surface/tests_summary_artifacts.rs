use super::*;

#[test]
fn mvp_check_covers_public_artifacts_and_local_design_partner_evaluation() {
    let summary = local_mvp_summary();

    let correctness_report =
        assert_passed_criterion(&summary, "public_correctness_report_available");
    assert!(correctness_report
        .evidence
        .contains("cargo test --workspace"));

    let positioning =
        assert_passed_criterion(&summary, "source_safety_verified_replication_positioning");
    assert_contains_all(
        &positioning.evidence,
        &[
            "PostgreSQL fleets",
            "read-only source-safety/failover-slot proof surface current=true",
        ],
    );
    assert_contains_all(
        &positioning.proof_command,
        &["README.md", "pilot_package_source_safety.rs"],
    );

    let design_partner =
        assert_passed_criterion(&summary, "design_partner_can_evaluate_without_kafka");
    assert_contains_all(
        &design_partner.evidence,
        &["local-run-proof.md", "no-Kafka evaluation"],
    );
    assert_eq!(design_partner.proof_command, "make quickstart-proof-check");
    assert!(summary
        .next_commands
        .contains(&"trellara chaos run".to_string()));
}
