use std::path::Path;

pub(crate) fn pilot_package_next_commands(config_path: &Path, output: &Path) -> Vec<String> {
    vec![
        format!(
            "trellara check --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara quickstart --config {} --check",
            config_path.display()
        ),
        format!(
            "trellara pilot-guide --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara pilot-scorecard --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara pilot-evidence --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara pilot evidence-template --config {} --output {}/live-evidence --format text",
            config_path.display(),
            output.display()
        ),
        format!(
            "trellara pilot evidence-check --config {} --evidence-dir {}/live-evidence --format text",
            config_path.display(),
            output.display()
        ),
        format!("trellara evaluate --config {} --format text", config_path.display()),
        format!(
            "trellara schema ddl-plan --config {} --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe --format text",
            config_path.display()
        ),
        format!(
            "trellara fleet report --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara fleet scorecard --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara fleet evidence-plan --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara consistency --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara performance --config {} --format text",
            config_path.display()
        ),
        format!(
            "trellara identity-audit --config {} --format text",
            config_path.display()
        ),
        format!("trellara semantics --config {}", config_path.display()),
        format!("trellara lake ddl --config {}", config_path.display()),
        format!("trellara lake epoch --config {}", config_path.display()),
        format!(
            "trellara lake fanin verify --config {} --stream-epoch {}/lake-epoch.json --lake-epoch {}/lake-epoch.json --accept-complete-with-gaps",
            config_path.display(),
            output.display(),
            output.display()
        ),
        format!(
            "trellara lake writer-plan --config {} --file {}/sample-envelope.pb --format text",
            config_path.display(),
            output.display()
        ),
        format!(
            "trellara lake fanin run --config {} --file {}/sample-envelope.pb --format text",
            config_path.display(),
            output.display()
        ),
        format!(
            "trellara lake spark-template current-state --config {} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text",
            config_path.display()
        ),
        format!(
            "trellara lake spark-template scd2 --config {} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text",
            config_path.display()
        ),
        format!(
            "trellara lake spark-template maintenance --config {} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text",
            config_path.display()
        ),
        format!(
            "trellara lake spark-template dashboard --config {} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text",
            config_path.display()
        ),
        format!(
            "trellara fleet control-plane --config {}",
            config_path.display()
        ),
        format!(
            "trellara status --config {} --view diagnostics --format text",
            config_path.display()
        ),
        format!(
            "trellara evidence-registry --package {} --format text",
            output.display()
        ),
    ]
}
