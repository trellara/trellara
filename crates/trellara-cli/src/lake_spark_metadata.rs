use crate::LakeSparkTemplateKind;

pub(crate) fn lake_spark_template_kind_label(template: LakeSparkTemplateKind) -> &'static str {
    match template {
        LakeSparkTemplateKind::CurrentState => "current-state",
        LakeSparkTemplateKind::Scd2 => "scd2",
        LakeSparkTemplateKind::Maintenance => "maintenance",
        LakeSparkTemplateKind::Dashboard => "dashboard",
    }
}

pub(crate) fn lake_spark_template_sql_file_name(template: LakeSparkTemplateKind) -> &'static str {
    match template {
        LakeSparkTemplateKind::CurrentState => "spark-current-state.sql",
        LakeSparkTemplateKind::Scd2 => "spark-scd2.sql",
        LakeSparkTemplateKind::Maintenance => "spark-maintenance.sql",
        LakeSparkTemplateKind::Dashboard => "spark-completeness-dashboard.sql",
    }
}

pub(crate) fn template_requires_primary_key(template: LakeSparkTemplateKind) -> bool {
    matches!(
        template,
        LakeSparkTemplateKind::CurrentState | LakeSparkTemplateKind::Scd2
    )
}

pub(crate) fn lake_spark_idempotency_rule(template: LakeSparkTemplateKind) -> String {
    match template {
        LakeSparkTemplateKind::CurrentState | LakeSparkTemplateKind::Scd2 => {
            "deduplicate and merge by source id, relation, record key, commit LSN, total order, and Trellara idempotency key"
                .to_string()
        }
        LakeSparkTemplateKind::Maintenance => {
            "optimize and expire snapshots only after verified epoch visibility; reruns are safe because table maintenance is metadata/file-layout only"
                .to_string()
        }
        LakeSparkTemplateKind::Dashboard => {
            "read-only completeness queries are safe to rerun and must keep complete_with_gaps acceptance explicit in run evidence"
                .to_string()
        }
    }
}
