use super::*;

pub(super) fn assert_pilot_package_artifacts(summary: &PilotPackageSummary) {
    assert_eq!(summary.artifact_count, 51);

    for (path_suffix, purpose_fragment) in [
        ("quickstart-readiness.txt", "readiness checks"),
        ("executive-evidence.md", "executive evidence brief"),
        (
            "enterprise-evaluation.txt",
            "buyer-facing enterprise readiness",
        ),
        (
            "enterprise-evaluation.json",
            "machine-readable enterprise readiness",
        ),
        ("schema-ddl-plan.json", "schema-barrier release gates"),
        ("schema-ddl-apply-plan.json", "safe SQL"),
        ("schema-ddl-envelope-plan.json", "propagation boundary"),
        ("schema-ddl-envelope-plan.json", "policy digest"),
        ("schema-ddl-envelope-plan.json", "replay-envelope proof"),
        ("ddl-barrier-status.json", "DDL barrier status"),
        ("ddl-release-proof.json", "required sink ACK evidence"),
        ("local-run-proof.md", "human-readable evidence slot"),
        (
            "source-safety-checklist.md",
            "source safety diagnostic checklist",
        ),
        ("proof-bundle.md", "single-review proof chain"),
        ("deployment-guide.md", "partner-specific deployment guide"),
        ("operational-burden-notes.md", "operational burden notes"),
        ("feature-pull-list.md", "control-plane"),
        ("fleet-report.txt", "fleet topology"),
        ("fleet-scorecard.txt", "fleet readiness"),
        ("fleet-evidence-plan.txt", "live-evidence command plan"),
        (
            "consistency-contract.json",
            "flow-level consistency contract",
        ),
        ("performance-envelope.json", "performance envelope"),
        ("identity-audit.json", "replica-identity"),
        ("consumer-semantics.json", "consumer semantics matrix"),
        ("lake-ddl.json", "raw CDC and epoch metadata"),
        ("lake-epoch.json", "fleet fan-in epoch completeness"),
        ("lake-verify.json", "stream-to-lake epoch"),
        ("lake-completeness.json", "Iceberg completeness evidence"),
        ("lake-writer-plan.json", "raw CDC append-file"),
        ("sample-envelope.pb", "deterministic transaction envelope"),
        ("lake-fanin-run.json", "dry-run verdict"),
        ("spark-current-state.sql", "current-state"),
        ("spark-current-state.py", "template_sha256"),
        ("spark-scd2.sql", "SCD2"),
        ("spark-scd2.py", "template_sha256"),
        ("spark-maintenance.sql", "compacting"),
        ("spark-maintenance.py", "template_sha256"),
        ("spark-completeness-dashboard.sql", "completeness"),
        ("spark-completeness-dashboard.py", "template_sha256"),
        ("spark-golden-fixture.json", "golden outputs"),
        ("fleet-control-plane.json", "control-plane pull report"),
        ("diagnostics.txt", "support-ready diagnostics"),
        ("diagnostics.json", "machine-readable diagnostics"),
        (
            "correctness-report.html",
            "deterministic correctness report",
        ),
        (
            "live-evidence/README.md",
            "collecting live scorecard evidence",
        ),
        ("live-evidence/collect.sh", "gate evidence artifacts"),
    ] {
        assert_artifact(summary, path_suffix, purpose_fragment);
    }
}

fn assert_artifact(summary: &PilotPackageSummary, path_suffix: &str, purpose_fragment: &str) {
    assert!(
        summary.artifacts.iter().any(|artifact| {
            artifact.path.ends_with(path_suffix)
                && artifact.purpose.contains(purpose_fragment)
                && artifact.byte_count > 0
                && artifact.sha256.len() == 64
        }),
        "missing artifact with path suffix {path_suffix:?} and purpose fragment {purpose_fragment:?}"
    );
}
