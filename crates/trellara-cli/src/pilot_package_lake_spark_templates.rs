use super::LakeCompletenessSparkTemplate;
use crate::LakeSparkTemplateKind;

pub(crate) fn spark_templates(spark_consumption_gate: &str) -> Vec<LakeCompletenessSparkTemplate> {
    [
        (
            LakeSparkTemplateKind::CurrentState,
            "spark-current-state.sql",
        ),
        (LakeSparkTemplateKind::Scd2, "spark-scd2.sql"),
        (LakeSparkTemplateKind::Maintenance, "spark-maintenance.sql"),
        (
            LakeSparkTemplateKind::Dashboard,
            "spark-completeness-dashboard.sql",
        ),
    ]
    .into_iter()
    .map(|(kind, artifact)| LakeCompletenessSparkTemplate {
        kind: spark_template_kind_label(kind).to_string(),
        artifact: artifact.to_string(),
        gate: spark_consumption_gate.to_string(),
    })
    .collect()
}

fn spark_template_kind_label(kind: LakeSparkTemplateKind) -> &'static str {
    match kind {
        LakeSparkTemplateKind::CurrentState => "current_state",
        LakeSparkTemplateKind::Scd2 => "scd2",
        LakeSparkTemplateKind::Maintenance => "maintenance",
        LakeSparkTemplateKind::Dashboard => "dashboard",
    }
}
