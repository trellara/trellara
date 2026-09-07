use crate::{FlowAlertSeverity, SourceSafetyGrade};

pub(crate) fn source_safety_grade_label(grade: SourceSafetyGrade) -> &'static str {
    match grade {
        SourceSafetyGrade::A => "A",
        SourceSafetyGrade::B => "B",
        SourceSafetyGrade::C => "C",
        SourceSafetyGrade::D => "D",
        SourceSafetyGrade::F => "F",
    }
}

pub(crate) fn optional_bool_label(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

pub(crate) fn flow_alert_severity_label(severity: FlowAlertSeverity) -> &'static str {
    match severity {
        FlowAlertSeverity::Warning => "warning",
        FlowAlertSeverity::Critical => "critical",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_safety_grade_labels_match_external_report_contract() {
        assert_eq!(source_safety_grade_label(SourceSafetyGrade::A), "A");
        assert_eq!(source_safety_grade_label(SourceSafetyGrade::B), "B");
        assert_eq!(source_safety_grade_label(SourceSafetyGrade::C), "C");
        assert_eq!(source_safety_grade_label(SourceSafetyGrade::D), "D");
        assert_eq!(source_safety_grade_label(SourceSafetyGrade::F), "F");
    }

    #[test]
    fn optional_bool_labels_preserve_unknown_slot_state() {
        assert_eq!(optional_bool_label(Some(true)), "true");
        assert_eq!(optional_bool_label(Some(false)), "false");
        assert_eq!(optional_bool_label(None), "unknown");
    }

    #[test]
    fn alert_severity_labels_match_text_surfaces() {
        assert_eq!(
            flow_alert_severity_label(FlowAlertSeverity::Warning),
            "warning"
        );
        assert_eq!(
            flow_alert_severity_label(FlowAlertSeverity::Critical),
            "critical"
        );
    }
}
