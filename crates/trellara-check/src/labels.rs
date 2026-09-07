use crate::{CheckGrade, CheckSeverity, CheckStatus};

pub(crate) fn grade_label(grade: CheckGrade) -> &'static str {
    match grade {
        CheckGrade::A => "A",
        CheckGrade::B => "B",
        CheckGrade::C => "C",
        CheckGrade::D => "D",
        CheckGrade::F => "F",
    }
}

pub(crate) fn status_label(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Healthy => "healthy",
        CheckStatus::Degraded => "degraded",
        CheckStatus::Blocked => "blocked",
    }
}

pub(crate) fn severity_label(severity: CheckSeverity) -> &'static str {
    match severity {
        CheckSeverity::Warning => "warning",
        CheckSeverity::Critical => "critical",
    }
}

pub(crate) fn optional_bool_label(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

pub(crate) fn format_duration(seconds: i64) -> String {
    let seconds = seconds.max(0);
    if seconds >= 60 * 60 {
        format!("~{} hours", ((seconds + 30 * 60) / (60 * 60)).max(1))
    } else if seconds >= 60 {
        format!("~{} minutes", ((seconds + 30) / 60).max(1))
    } else {
        format!("~{} seconds", seconds)
    }
}

pub(crate) fn display_optional_duration(seconds: Option<i64>) -> String {
    seconds
        .map(format_duration)
        .unwrap_or_else(|| "none".to_string())
}

pub(crate) fn display_optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duration_labels_match_incident_language() {
        assert_eq!(format_duration(50_400), "~14 hours");
        assert_eq!(format_duration(600), "~10 minutes");
        assert_eq!(format_duration(4), "~4 seconds");
    }
}
