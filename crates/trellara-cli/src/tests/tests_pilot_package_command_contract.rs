use super::*;

pub(super) fn assert_pilot_package_next_commands(
    summary: &PilotPackageSummary,
    config_path: &std::path::Path,
    output_path: &std::path::Path,
) {
    let config = config_path.display();
    let output = output_path.display();
    let expected_commands = [
        format!("trellara check --config {config} --format text"),
        format!("trellara pilot-guide --config {config} --format text"),
        format!("trellara pilot-scorecard --config {config} --format text"),
        format!("trellara pilot-evidence --config {config} --format text"),
        format!(
            "trellara pilot evidence-template --config {config} --output {output}/live-evidence --format text"
        ),
        format!(
            "trellara pilot evidence-check --config {config} --evidence-dir {output}/live-evidence --format text"
        ),
        format!("trellara evaluate --config {config} --format text"),
        format!(
            "trellara schema ddl-plan --config {config} --change add_nullable_column:public.sales.discount_code:text --apply-mode auto-safe --format text"
        ),
        format!("trellara fleet report --config {config} --format text"),
        format!("trellara fleet scorecard --config {config} --format text"),
        format!("trellara fleet evidence-plan --config {config} --format text"),
        format!("trellara consistency --config {config} --format text"),
        format!("trellara performance --config {config} --format text"),
        format!("trellara identity-audit --config {config} --format text"),
        format!("trellara semantics --config {config}"),
        format!("trellara lake ddl --config {config}"),
        format!("trellara lake epoch --config {config}"),
        format!(
            "trellara lake fanin verify --config {config} --stream-epoch {output}/lake-epoch.json --lake-epoch {output}/lake-epoch.json --accept-complete-with-gaps"
        ),
        format!(
            "trellara lake writer-plan --config {config} --file {output}/sample-envelope.pb --format text"
        ),
        format!(
            "trellara lake fanin run --config {config} --file {output}/sample-envelope.pb --format text"
        ),
        format!(
            "trellara lake spark-template current-state --config {config} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text"
        ),
        format!(
            "trellara lake spark-template scd2 --config {config} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text"
        ),
        format!(
            "trellara lake spark-template maintenance --config {config} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text"
        ),
        format!(
            "trellara lake spark-template dashboard --config {config} --table public.sales --epoch-id epoch-2026-08-16T00 --accept-complete-with-gaps --format text"
        ),
        format!("trellara fleet control-plane --config {config}"),
        format!("trellara status --config {config} --view diagnostics --format text"),
        format!("trellara evidence-registry --package {output} --format text"),
    ];

    for command in expected_commands {
        assert!(
            summary.next_commands.contains(&command),
            "missing pilot package next command: {command}"
        );
    }
}
