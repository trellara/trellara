use crate::MvpReadinessCriterion;

pub(crate) fn mvp_readiness_next_commands(
    config_display: &str,
    criteria: &[MvpReadinessCriterion],
    ready: bool,
) -> Vec<String> {
    if ready {
        vec![
            format!("trellara quickstart --config {config_display} --check --format text"),
            "trellara chaos run".to_string(),
            "make verify-correctness-report".to_string(),
            format!("trellara pilot-package --config {config_display}"),
        ]
    } else {
        criteria
            .iter()
            .filter(|criterion| !criterion.passed)
            .map(|criterion| criterion.proof_command.clone())
            .collect()
    }
}

pub(crate) fn mvp_readiness_priority_next_commands(
    config_display: &str,
    criteria: &[MvpReadinessCriterion],
    ready: bool,
) -> Vec<String> {
    if ready {
        vec![
            format!("trellara quickstart --config {config_display} --check --format text"),
            format!("trellara pilot-package --config {config_display}"),
        ]
    } else {
        criteria
            .iter()
            .find(|criterion| !criterion.passed)
            .map(|criterion| vec![criterion.proof_command.clone()])
            .unwrap_or_default()
    }
}
