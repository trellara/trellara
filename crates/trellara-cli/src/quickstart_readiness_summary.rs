use crate::{
    QuickstartReadinessCheck, QuickstartReadinessSummary, QUICKSTART_ESTIMATED_MINUTES,
    QUICKSTART_TIME_BUDGET_MINUTES,
};

impl QuickstartReadinessSummary {
    pub(crate) fn from_checks(config: String, checks: Vec<QuickstartReadinessCheck>) -> Self {
        let passed_check_count = checks.iter().filter(|check| check.passed).count();
        let ready = passed_check_count == checks.len();
        let evidence_bundle = ready.then(|| "target/trellara-quickstart-evidence".to_string());
        let recovery_command = ready
            .then(|| format!("trellara status --config {config} --view diagnostics --format text"));
        let next_commands = if ready {
            vec![
                format!("trellara check --config {config} --format text"),
                format!("trellara preflight --config {config}"),
                format!(
                    "trellara run --local --verify --format text --config {config} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"
                ),
                format!("trellara status --config {config} --view report --format text"),
                format!(
                    "trellara pilot-package --config {config} --output target/trellara-quickstart-evidence"
                ),
            ]
        } else {
            checks
                .iter()
                .filter_map(|check| check.fix.clone())
                .collect::<Vec<_>>()
        };

        Self {
            config,
            ready,
            estimated_minutes: ready.then_some(QUICKSTART_ESTIMATED_MINUTES),
            time_budget_minutes: QUICKSTART_TIME_BUDGET_MINUTES,
            evidence_bundle,
            recovery_command,
            check_count: checks.len(),
            passed_check_count,
            checks,
            next_commands,
        }
    }
}

impl QuickstartReadinessCheck {
    pub(crate) fn passed(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            passed: true,
            message: message.into(),
            fix: None,
        }
    }

    pub(crate) fn failed(
        code: impl Into<String>,
        message: impl Into<String>,
        fix: Option<String>,
    ) -> Self {
        Self {
            code: code.into(),
            passed: false,
            message: message.into(),
            fix,
        }
    }
}
