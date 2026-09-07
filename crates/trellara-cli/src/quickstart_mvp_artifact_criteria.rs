use std::path::Path;

use crate::quickstart_artifacts::correctness_report_workflow_publishes_tested_report;
use crate::quickstart_design_partner_artifacts::local_design_partner_artifacts_are_current;
use crate::quickstart_protocol_artifacts::protocol_property_tests_are_current;
use crate::quickstart_source_safety_artifacts::source_safety_enterprise_artifacts_are_current;
use crate::{
    chaos_has_scenario, product_positioning_artifacts_are_current,
    schema_ddl_propagation_artifacts_are_current, ChaosRunSummary, MvpReadinessCriterion,
    StreamConfig, TrellaraConfig, CORRECTNESS_REPORT_ARTIFACT,
};

pub(crate) fn build_artifact_readiness_criteria(
    config: &TrellaraConfig,
    config_display: &str,
    repository_root: &Path,
    chaos: &ChaosRunSummary,
) -> Vec<MvpReadinessCriterion> {
    vec![
        MvpReadinessCriterion::new(
            "ddl_propagation_contract_packaged",
            schema_ddl_propagation_artifacts_are_current(repository_root),
            "schema-ddl-plan.json, schema-ddl-apply-plan.json, schema-ddl-envelope-plan.json, ddl-barrier-status.json, and ddl-release-proof.json package schema-barrier propagation, retry-safe additive target SQL, a reviewed target Postgres transaction script with plan and statement digests, runtime DDL envelope classification, CDC transaction-boundary proof, propagation boundary, propagation decisions, propagation policy digest, target/lake/Spark sink ACKs with ack evidence, DML hold, release blockers with stable release_blocker_codes, release proof, partition visibility pause semantics, and ddl-barrier record/ack/status commands".to_string(),
            format!(
                "trellara schema ddl-plan --config {config_display} --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe --format json && trellara schema ddl-apply-plan --config {config_display} --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe --format json && trellara schema ddl-envelope-plan --config {config_display} --file <schema-ddl-envelope.pb> --format json && trellara schema ddl-barrier status --config {config_display} --barrier-id <ddl-barrier-id> --format json && cargo test -p trellara-apply-postgres --lib target_ddl_transaction_plan_accepts_retry_safe_additive_sql && make quickstart-proof-check"
            ),
        ),
        MvpReadinessCriterion::new(
            "protocol_property_tests_cover_invariants",
            chaos_has_scenario(chaos, "protocol_property_chunk_manifest_reconstruction")
                && protocol_property_tests_are_current(repository_root),
            format!(
                "protocol property scenario covered={}; modular envelope/idempotency/LSN/strict chunk/partitioned/missing/duplicate proptests current={}",
                chaos_has_scenario(chaos, "protocol_property_chunk_manifest_reconstruction"),
                protocol_property_tests_are_current(repository_root)
            ),
            "cargo test -p trellara-protocol property".to_string(),
        ),
        MvpReadinessCriterion::new(
            "failure_matrix_covered",
            chaos.passed && chaos.scenario_count == chaos.covered_scenarios && chaos.simulations_passed,
            format!(
                "{} of {} scenarios covered; {} deterministic simulations pass",
                chaos.covered_scenarios, chaos.scenario_count, chaos.simulation_count
            ),
            "trellara chaos run".to_string(),
        ),
        MvpReadinessCriterion::new(
            "public_correctness_report_available",
            repository_root.join(CORRECTNESS_REPORT_ARTIFACT).exists()
                && correctness_report_workflow_publishes_tested_report(repository_root),
            format!(
                "{CORRECTNESS_REPORT_ARTIFACT} exists and .github/workflows/correctness-report.yml runs cargo test --workspace before publishing the site"
            ),
            "make verify-correctness-report".to_string(),
        ),
        MvpReadinessCriterion::new(
            "source_safety_verified_replication_positioning",
            product_positioning_artifacts_are_current(repository_root)
                && source_safety_enterprise_artifacts_are_current(repository_root),
            format!(
                "README positions Trellara around source safety, verified replication, correctness reports, and PostgreSQL fleets; read-only source-safety/failover-slot proof surface current={}",
                source_safety_enterprise_artifacts_are_current(repository_root)
            ),
            "rg -i \"source-safety|verified replication|correctness report|postgres|fleet|read-only|failover slot\" README.md crates/trellara-cli/src/pilot_package_source_safety.rs".to_string(),
        ),
        MvpReadinessCriterion::new(
            "design_partner_can_evaluate_without_kafka",
            matches!(config.stream, StreamConfig::Local { .. })
                && config.target.is_some()
                && local_design_partner_artifacts_are_current(repository_root),
            "stream.kind=local with target.database_url configured and pilot package exports enterprise-evaluation.txt, local-run-proof.md, fleet-report.txt, and manifest integrity checks for no-Kafka evaluation".to_string(),
            "make quickstart-proof-check".to_string(),
        ),
    ]
}
