use std::fs;
use std::path::Path;

pub(super) fn is_current(repository_root: &Path) -> bool {
    let cli_src = repository_root
        .join("crates")
        .join("trellara-cli")
        .join("src");
    read_required_sources(&cli_src)
        .map(|source| append_optional_sources(source, &cli_src))
        .and_then(|source| read_required_types(&cli_src).map(|types| (source, types)))
        .is_ok_and(|(source, types)| types_are_current(&types) && sources_are_current(&source))
}

fn read_required_sources(cli_src: &Path) -> std::io::Result<String> {
    let mut source = fs::read_to_string(cli_src.join("ddl.rs"))?;
    for file in ["ddl_change.rs", "ddl_sql.rs", "ddl_propagation.rs"] {
        source.push_str(&fs::read_to_string(cli_src.join(file))?);
    }
    Ok(source)
}

fn append_optional_sources(mut source: String, cli_src: &Path) -> String {
    for file in OPTIONAL_SOURCE_FILES {
        if let Ok(extra) = fs::read_to_string(cli_src.join(file)) {
            source.push_str(&extra);
        }
    }
    source
}

fn read_required_types(cli_src: &Path) -> std::io::Result<String> {
    let mut types = fs::read_to_string(cli_src.join("ddl_types.rs"))?;
    types.push_str(&fs::read_to_string(
        cli_src.join("ddl_propagation_types.rs"),
    )?);
    Ok(types)
}

fn types_are_current(types: &str) -> bool {
    contains_all(
        types,
        &[
            "struct DdlPropagationPlan",
            "struct DdlPlanBlocker",
            "struct DdlApplyPlanSummary",
            "struct DdlPropagationSink",
            "struct DdlPropagationPhase",
            "struct DdlReleaseGate",
            "release_gates",
            "dml_after_barrier_held",
            "requires_global_partition_pause",
            "ack_evidence",
            "cdc_transaction_boundary",
            "release_impact",
        ],
    )
}

fn sources_are_current(source: &str) -> bool {
    contains_all(
        source,
        &[
            "ack_commands",
            "DdlPropagationSinkKind::RawCdcLake",
            "DdlPropagationSinkKind::SparkDerivedView",
            "DdlPropagationSinkKind::PartitionVisibility",
            "fn ddl_release_gates",
            "DdlApplyPlanSummary",
            "ddl_plan_blockers",
            "post_ddl_dml_release",
            "fn ddl_target_postgres_sql",
            "ADD COLUMN",
            "ALTER COLUMN",
            "DdlEnvelopePlanSummary",
            "propagation_boundary",
            "propagation_decisions",
            "propagation_policy_sha256",
            "ddl_events stripped",
            "DdlEnvelopeReplaySummary",
            "dml_replay_after_ddl_barrier",
            "target_ddl_apply_plan_from_envelope",
            "target_ddl_barrier_from_envelope_with_requirements",
            "partition-watermarks shows all partitions at or beyond the barrier LSN",
            "post_ddl_release_gate_satisfied",
            "ack_commands_are_executable",
            "command_is_executable",
            "command_collection_is_valid",
            "ack_evidence_has_valid_sink_lsns",
            "evidence_item_is_valid",
            "evidence_covers_required_sinks",
            "cdc_transaction_boundary",
            "\"ack_commands\"",
            "\"ack_evidence\"",
        ],
    )
}

fn contains_all(contents: &str, fragments: &[&str]) -> bool {
    fragments.iter().all(|fragment| contents.contains(fragment))
}

const OPTIONAL_SOURCE_FILES: &[&str] = &[
    "ddl_propagation_actions.rs",
    "ddl_propagation_boundary.rs",
    "ddl_release_gates.rs",
    "ddl_apply_plan.rs",
    "ddl_envelope_barrier_plan.rs",
    "ddl_envelope_plan.rs",
    "ddl_envelope_summaries.rs",
    "ddl_envelope_target.rs",
    "ddl_envelope_replay.rs",
    "ddl_envelope_render.rs",
    "pilot_package_ddl_release.rs",
    "pilot_evidence_ddl_markers.rs",
    "pilot_evidence_ddl_markers/json.rs",
    "pilot_evidence_ddl_markers/ack.rs",
    "pilot_evidence_ddl_markers/ack/command.rs",
    "pilot_evidence_ddl_markers/ack/evidence.rs",
    "pilot_evidence_ddl_markers/ack/sinks.rs",
];
