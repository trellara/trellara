use super::{acceptable_status, meaningful_str};

pub(super) fn has_no_critical_findings(lower: &str) -> bool {
    lower.contains("trellara source safety")
        && lower.contains("findings:")
        && findings_are_explicit(lower)
        && !lower.contains("[critical]")
        && !lower.contains("severity=critical")
        && !lower.contains("\"severity\":\"critical\"")
        && !lower.contains("\"severity\": \"critical\"")
}

pub(super) fn status_is_acceptable(lower: &str) -> bool {
    lower
        .lines()
        .filter_map(|line| line.trim().strip_prefix("status:"))
        .map(str::trim)
        .any(acceptable_status)
}

pub(super) fn identity_present(lower: &str) -> bool {
    identity_field_present(lower, "source") && identity_field_present(lower, "dataset")
}

fn findings_are_explicit(lower: &str) -> bool {
    let Some((_, findings)) = lower.split_once("findings:") else {
        return false;
    };

    findings.lines().any(|line| {
        let finding = line.trim();
        finding.starts_with("- none") || finding.contains("[warning]")
    })
}

fn identity_field_present(lower: &str, key: &str) -> bool {
    lower.lines().any(|line| {
        let trimmed = line.trim();
        let value = trimmed
            .strip_prefix(&format!("{key}:"))
            .or_else(|| trimmed.strip_prefix(&format!("{key}_id=")));
        value.is_some_and(meaningful_str)
    })
}
