use std::fs;
use std::path::Path;

pub(crate) fn source_safety_enterprise_artifacts_are_current(repository_root: &Path) -> bool {
    source_safety_checklist_is_current(repository_root)
        && source_safety_output_is_read_only(repository_root)
        && source_slot_evidence_is_current(repository_root)
        && source_safety_proof_gate_is_current(repository_root)
}

fn source_safety_checklist_is_current(repository_root: &Path) -> bool {
    read_source(repository_root, "pilot_package_source_safety.rs").is_some_and(|source| {
        contains_all(
            &source,
            &[
                "## Read-Only Commands",
                "trellara-check <source-database-url>",
                "trellara check --config",
                "trellara check --database-url <source-database-url>",
                "--table <schema.table>",
                "trellara-check",
                "--format html",
                "source-safety.html",
                "--write-init",
                "slot `wal_status`",
                "`safe_wal_size`",
                "`failover`, `synced`",
                "`idle_replication_slot_timeout`",
                "transaction ID wraparound budget",
                "xmin horizon pinners",
                "Do not start capture against a customer source",
            ],
        )
    })
}

fn source_safety_output_is_read_only(repository_root: &Path) -> bool {
    let text_current = read_source(repository_root, "source_safety/text.rs")
        .is_some_and(|source| source.contains("read_only: {read_only}"));
    let render_current =
        read_source(repository_root, "source_safety/render.rs").is_some_and(|source| {
            source.contains("read_only: Some(summary.read_only)")
                && source.contains("read_only: None")
        });
    let html_current =
        read_source(repository_root, "source_safety/html/queries.rs").is_some_and(|source| {
            source.contains("Exact Read-Only Queries")
                && source.contains("source_safety_read_only_queries")
        });
    let direct_current = read_source(repository_root, "source_safety/direct.rs")
        .is_some_and(|source| source.contains("read_only: true"));

    text_current && render_current && html_current && direct_current
}

fn source_slot_evidence_is_current(repository_root: &Path) -> bool {
    read_source(repository_root, "source_safety/slot/evidence.rs").is_some_and(|source| {
        contains_all(
            &source,
            &[
                "restart_lsn",
                "confirmed_flush_lsn",
                "wal_status",
                "safe_wal_size_bytes",
                "retained_wal_bytes",
                "invalidation_reason",
                "\"failover\"",
                "\"synced\"",
                "inactive_since",
                "idle_replication_slot_timeout",
            ],
        )
    })
}

fn source_safety_proof_gate_is_current(repository_root: &Path) -> bool {
    let makefile_current =
        fs::read_to_string(repository_root.join("Makefile")).is_ok_and(|makefile| {
            contains_all(
                &makefile,
                &[
                    "source-safety-checklist.md",
                    "trellara check --config",
                    "trellara-check",
                    "failover slot",
                    "wal_status",
                    "replica identity",
                ],
            )
        });
    let readme_current = fs::read_to_string(repository_root.join("docs").join("DESIGN.md"))
        .is_ok_and(|readme| {
            let readme = readme.to_ascii_lowercase();
            contains_all(
                &readme,
                &[
                    "read-only source-safety checklist",
                    "trellara-check",
                    "failover-slot evidence",
                    "source-safety.html",
                    "source-safety-checklist.md",
                ],
            ) && readme.contains("trellara-check <source-database-url>")
        });

    makefile_current && readme_current
}

fn read_source(repository_root: &Path, relative_path: &str) -> Option<String> {
    fs::read_to_string(
        repository_root
            .join("crates")
            .join("trellara-cli")
            .join("src")
            .join(relative_path),
    )
    .ok()
}

fn contains_all(source: &str, required: &[&str]) -> bool {
    required.iter().all(|needle| source.contains(needle))
}
