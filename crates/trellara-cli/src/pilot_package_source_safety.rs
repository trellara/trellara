use std::fmt::Write as _;
use std::path::Path;

use crate::TrellaraConfig;

pub(crate) fn render_source_safety_checklist_artifact(
    config: &TrellaraConfig,
    config_path: &Path,
) -> String {
    let config_display = config_path.display();
    let mut output = String::new();
    writeln!(&mut output, "# Trellara Source Safety Checklist").expect("write string");
    writeln!(&mut output).expect("write string");
    writeln!(&mut output, "config: {config_display}").expect("write string");
    writeln!(&mut output, "source: {}", config.source.id).expect("write string");
    writeln!(&mut output, "dataset: {}", config.dataset.id).expect("write string");
    writeln!(&mut output, "mode: {}", config.status_mode()).expect("write string");
    writeln!(&mut output, "publication: {}", config.source.publication).expect("write string");
    writeln!(&mut output, "slot: {}", config.source.slot).expect("write string");
    writeln!(
        &mut output,
        "expected_plugin: {}",
        config.source.capture.expected_plugin()
    )
    .expect("write string");
    writeln!(&mut output).expect("write string");

    output.push_str("## Read-Only Commands\n\n");
    writeln!(
        &mut output,
        "```sh\ntrellara check --config {config_display} --format text\n```"
    )
    .expect("write string");
    writeln!(
        &mut output,
        "\n```sh\ntrellara check --config {config_display} --format html --output source-safety.html\n```"
    )
    .expect("write string");
    output.push_str(
        "\nFor an ad-hoc source before writing config, use the standalone diagnostic with the source URL only:\n\n",
    );
    output.push_str(
        "```sh\ntrellara-check <source-database-url> --format html --output source-safety.html\n```\n",
    );
    output.push_str(
        "\nUse the full CLI path when the same inspection should prepare an init config:\n\n",
    );
    output.push_str(
        "```sh\ntrellara check --database-url <source-database-url> --table <schema.table> --format html --output source-safety.html\n```\n",
    );
    output.push_str(
        "\nWhen the report has no critical blockers, materialize the evaluation config from the same inspection:\n\n",
    );
    writeln!(
        &mut output,
        "```sh\ntrellara check --database-url <source-database-url> --table <schema.table> --format text --write-init {config_display} --target-database-url <target-database-url>\n```"
    )
    .expect("write string");

    output.push_str("\n## Evidence To Retain\n\n");
    output.push_str("- `score`, `grade`, `critical_blockers`, and `recommended_actions` from the source-safety report.\n");
    output.push_str("- `source-safety.html` from `trellara-check` for ticket or manager review, including findings, evidence, recovery actions, and exact read-only SQL.\n");
    output.push_str("- Table CDC readiness for primary keys, replica identity, unchanged TOAST columns, and unsupported relations.\n");
    output.push_str("- Replication slot plugin, slot `wal_status`, `safe_wal_size`, `invalidation_reason`, restart LSN, and confirmed flush LSN.\n");
    output.push_str("- Postgres failover slot posture: `failover`, `synced`, inactive-since metadata, and `idle_replication_slot_timeout` when the server exposes them.\n");
    output.push_str("- Projected WAL headroom from slot `safe_wal_size`, fallback WAL retention risk against `source.wal_retention_warn_bytes`, transaction ID wraparound budget, xmin horizon pinners, and any logical replication subscription conflicts on the source or subscriber.\n");
    output.push_str("- Generated `init_recommendation`, including the table list and target URL placeholder used for the first brokerless proof.\n");

    output.push_str("\n## Tables In Scope\n\n");
    if config.dataset.tables.is_empty() {
        output.push_str(
            "- none configured; source-safety should discover candidate tables before init\n",
        );
    } else {
        for table in &config.dataset.tables {
            writeln!(
                &mut output,
                "- `{}`: require primary key or safe replica identity before CDC approval",
                table.relation_id().display_name()
            )
            .expect("write string");
        }
    }

    output.push_str("\n## Decision Rule\n\n");
    output.push_str("Do not start capture against a customer source until source-safety has no critical blockers, the slot uses the expected plugin, WAL posture is not lost or unreserved, failover slot metadata is explicit for HA sources, and every replicated table has a primary key or reviewed replica identity.\n");
    output
}
