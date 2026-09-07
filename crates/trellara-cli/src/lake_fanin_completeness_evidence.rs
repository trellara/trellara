use crate::{
    lake_spark_template_kind_label, lake_spark_template_sql_file_name, LakeDdlSummary,
    LakeEpochSourceWatermark, LakeFaninCompletenessArgs, LakeFaninCompletenessSourceStateCounts,
    LakeFaninCompletenessSparkTemplate, LakeFaninCompletenessTableRef, LakeSparkTemplateKind,
};

pub(super) fn source_state_counts(
    source_rows: &[LakeEpochSourceWatermark],
) -> LakeFaninCompletenessSourceStateCounts {
    let mut counts = LakeFaninCompletenessSourceStateCounts::default();
    for source in source_rows {
        match source.state.as_str() {
            "complete" => counts.complete += 1,
            "lagging" => counts.lagging += 1,
            "missing" => counts.missing += 1,
            "quarantined" => counts.quarantined += 1,
            "reseeding" => counts.reseeding += 1,
            _ => counts.unknown += 1,
        }
    }
    counts
}

pub(super) fn ddl_table_refs(
    ddl: &LakeDdlSummary,
    raw_cdc: bool,
) -> Vec<LakeFaninCompletenessTableRef> {
    ddl.tables
        .iter()
        .filter(|table| (table.materialization == "raw_cdc_append_only") == raw_cdc)
        .map(|table| LakeFaninCompletenessTableRef {
            materialization: table.materialization.clone(),
            table_name: table.table_name.clone(),
        })
        .collect()
}

pub(super) fn spark_templates(
    args: &LakeFaninCompletenessArgs,
    epoch_id: &str,
    gate: &str,
) -> Vec<LakeFaninCompletenessSparkTemplate> {
    [
        LakeSparkTemplateKind::CurrentState,
        LakeSparkTemplateKind::Scd2,
        LakeSparkTemplateKind::Maintenance,
        LakeSparkTemplateKind::Dashboard,
    ]
    .into_iter()
    .map(|template| LakeFaninCompletenessSparkTemplate {
        kind: lake_spark_template_kind_label(template).to_string(),
        artifact: lake_spark_template_sql_file_name(template).to_string(),
        command: spark_template_command(args, template, epoch_id),
        gate: gate.to_string(),
    })
    .collect()
}

pub(super) fn proof_artifacts(args: &LakeFaninCompletenessArgs) -> Vec<String> {
    vec![
        args.stream_epoch.display().to_string(),
        args.lake_epoch.display().to_string(),
        "lake-verify.json".to_string(),
        "lake-completeness.json".to_string(),
        "spark-current-state.sql".to_string(),
        "spark-scd2.sql".to_string(),
        "spark-maintenance.sql".to_string(),
        "spark-completeness-dashboard.sql".to_string(),
    ]
}

pub(super) fn proof_commands(args: &LakeFaninCompletenessArgs, epoch_id: &str) -> Vec<String> {
    let mut verify_command = format!(
        "trellara lake fanin verify --config {} --stream-epoch {} --lake-epoch {}",
        args.config.display(),
        args.stream_epoch.display(),
        args.lake_epoch.display()
    );
    let mut completeness_command = format!(
        "trellara lake fanin completeness --config {} --stream-epoch {} --lake-epoch {} --format json",
        args.config.display(),
        args.stream_epoch.display(),
        args.lake_epoch.display()
    );
    if args.accept_complete_with_gaps {
        verify_command.push_str(" --accept-complete-with-gaps");
        completeness_command.push_str(" --accept-complete-with-gaps");
    }
    vec![
        verify_command,
        completeness_command,
        format!(
            "trellara lake spark-template dashboard --config {} --epoch-id {}{} --format text",
            args.config.display(),
            epoch_id,
            if args.accept_complete_with_gaps {
                " --accept-complete-with-gaps"
            } else {
                ""
            }
        ),
    ]
}

fn spark_template_command(
    args: &LakeFaninCompletenessArgs,
    template: LakeSparkTemplateKind,
    epoch_id: &str,
) -> String {
    let mut command = format!(
        "trellara lake spark-template {} --config {} --epoch-id {}",
        lake_spark_template_kind_label(template),
        args.config.display(),
        epoch_id
    );
    if args.accept_complete_with_gaps {
        command.push_str(" --accept-complete-with-gaps");
    }
    command.push_str(" --format text");
    command
}
