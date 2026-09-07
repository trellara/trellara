use crate::{
    QuickstartArgs, QuickstartCommand, QuickstartSummary, QUICKSTART_ESTIMATED_MINUTES,
    QUICKSTART_TIME_BUDGET_MINUTES,
};

impl QuickstartSummary {
    pub(crate) fn from_args(args: &QuickstartArgs) -> Self {
        let config = args.config.display().to_string();
        let evidence_bundle = "target/trellara-quickstart-evidence".to_string();
        let recovery_command =
            format!("trellara status --config {config} --view diagnostics --format text");
        let commands = vec![
            QuickstartCommand::new(
                1,
                "trellara dev up",
                "start the no-broker source and target Postgres stack",
            ),
            QuickstartCommand::new(
                2,
                format!(
                    "trellara init --source-database-url postgresql://trellara:trellara@localhost:55432/trellara_source --target-database-url postgresql://trellara:trellara@localhost:55433/trellara_target --source-id local-source --dataset-id retail-sales --publication trellara_retail --slot trellara_retail_slot --table public.sales --table public.sale_items --table public.payments --output {config} --evaluate --force"
                ),
                "write the brokerless local flow config",
            ),
            QuickstartCommand::new(
                3,
                format!("trellara check --config {config} --format text"),
                "inspect CDC safety read-only before starting capture",
            ),
            QuickstartCommand::new(
                4,
                format!("trellara preflight --config {config}"),
                "check source table, target compatibility, and strict chunk manifest settings before CDC starts",
            ),
            QuickstartCommand::new(
                5,
                format!("trellara run --local --verify --format text --config {config} --snapshot-run-id local-run-snapshot --max-transactions 100 --max-messages 100"),
                "create or reuse the initial snapshot handoff, then run bounded local relay, apply, and convergence verification without Kafka",
            ),
            QuickstartCommand::new(
                6,
                format!("trellara status --config {config} --view report --format text"),
                "produce the operator proof report for transaction boundaries, convergence, and recovery evidence",
            ),
            QuickstartCommand::new(
                7,
                format!("trellara pilot-package --config {config} --output {evidence_bundle}"),
                "write the shareable evidence bundle with manifest digests",
            ),
        ];
        let operator_reference_commands = vec![
            format!("trellara evaluate --config {config} --format text"),
            format!("trellara contract-test --config {config}"),
            format!("trellara pilot-guide --config {config} --format text"),
            format!("trellara pilot-scorecard --config {config} --format text"),
            format!("trellara pilot-evidence --config {config} --format text"),
            "trellara chaos report --output docs/correctness-report.html".to_string(),
        ];

        Self {
            config,
            objective:
                "run a no-broker local verified Postgres replication loop in under 10 minutes"
                    .to_string(),
            estimated_minutes: QUICKSTART_ESTIMATED_MINUTES,
            time_budget_minutes: QUICKSTART_TIME_BUDGET_MINUTES,
            evidence_bundle,
            recovery_command,
            operator_reference_commands,
            command_count: commands.len(),
            commands,
        }
    }
}
