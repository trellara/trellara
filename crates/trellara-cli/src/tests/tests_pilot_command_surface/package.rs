use super::*;

#[tokio::test]
async fn pilot_package_command_writes_summary_and_files() {
    let root = std::env::temp_dir().join(format!(
        "trellara-pilot-package-command-{}",
        std::process::id()
    ));
    let config_path = root.join("trellara.yml");
    let output_path = root.join("package");
    let correctness_report_path = root.join("correctness-report.html");
    fs::create_dir_all(&root).expect("create pilot package temp dir");
    fs::write(&config_path, local_stream_yaml()).expect("write pilot package config");
    fs::write(
        &correctness_report_path,
        "<html><body>Trellara correctness report</body></html>",
    )
    .expect("write correctness report");

    let output = execute(Cli {
        command: Command::PilotPackage(PilotPackageArgs {
            config: config_path.clone(),
            output: output_path.clone(),
            correctness_report: correctness_report_path.clone(),
        }),
    })
    .await
    .expect("pilot package output");

    assert_package_summary(&output);
    assert_package_readme(&output_path);
    assert_package_files(&output_path);
    assert_sample_envelope(&output_path);

    fs::remove_dir_all(root).expect("remove pilot package temp dir");
}

#[tokio::test]
async fn pilot_package_command_writes_partition_rebalance_plan_for_partitioned_mode() {
    let root = std::env::temp_dir().join(format!(
        "trellara-partitioned-pilot-package-command-{}",
        std::process::id()
    ));
    let config_path = root.join("partitioned.yml");
    let output_path = root.join("package");
    let correctness_report_path = root.join("correctness-report.html");
    fs::create_dir_all(&root).expect("create partitioned pilot package temp dir");
    fs::write(&config_path, partitioned_yaml()).expect("write partitioned config");
    fs::write(
        &correctness_report_path,
        "<html><body>Trellara correctness report</body></html>",
    )
    .expect("write correctness report");

    execute(Cli {
        command: Command::PilotPackage(PilotPackageArgs {
            config: config_path,
            output: output_path.clone(),
            correctness_report: correctness_report_path,
        }),
    })
    .await
    .expect("partitioned pilot package output");

    let plan = fs::read_to_string(output_path.join("partition-rebalance-plan.json")).expect("plan");
    assert!(plan.contains(r#""status": "Skewed""#));
    assert!(plan.contains(r#""runtime_movement_allowed": false"#));
    assert!(plan.contains(r#""visibility_contract""#));
    assert!(plan.contains(r#""recommended_moves""#));

    fs::remove_dir_all(root).expect("remove partitioned pilot package temp dir");
}

fn assert_package_summary(output: &str) {
    for marker in [
        "\"artifact_count\": 51",
        "quickstart-readiness.txt",
        "pilot-guide.json",
        "pilot-scorecard.json",
        "executive-evidence.md",
        "enterprise-evaluation.txt",
        "enterprise-evaluation.json",
        "schema-ddl-plan.json",
        "schema-ddl-apply-plan.json",
        "schema-ddl-envelope-plan.json",
        "ddl-barrier-status.json",
        "ddl-release-proof.json",
        "local-run-proof.md",
        "source-safety-checklist.md",
        "proof-bundle.md",
        "deployment-guide.md",
        "operational-burden-notes.md",
        "feature-pull-list.md",
        "fleet-report.txt",
        "fleet-scorecard.txt",
        "fleet-evidence-plan.txt",
        "consistency-contract.json",
        "performance-envelope.json",
        "identity-audit.json",
        "consumer-semantics.json",
        "lake-ddl.json",
        "lake-epoch.json",
        "lake-verify.json",
        "lake-completeness.json",
        "lake-writer-plan.json",
        "sample-envelope.pb",
        "lake-fanin-run.json",
        "spark-current-state.sql",
        "spark-current-state.py",
        "spark-scd2.sql",
        "spark-scd2.py",
        "spark-maintenance.sql",
        "spark-maintenance.py",
        "spark-completeness-dashboard.sql",
        "spark-completeness-dashboard.py",
        "spark-golden-fixture.json",
        "fleet-control-plane.json",
        "diagnostics.txt",
        "diagnostics.json",
        "correctness-report.html",
        "live-evidence/README.md",
        "live-evidence/collect.sh",
        "manifest.json",
        "\"sha256\"",
        "trellara evaluate --config",
        "trellara schema ddl-plan --config",
        "trellara fleet report --config",
        "trellara fleet scorecard --config",
        "trellara fleet evidence-plan --config",
        "trellara consistency --config",
        "trellara performance --config",
        "trellara identity-audit --config",
        "trellara semantics --config",
        "trellara lake ddl --config",
        "trellara lake fanin verify --config",
        "trellara lake writer-plan --config",
        "trellara lake spark-template current-state --config",
        "trellara lake spark-template scd2 --config",
        "trellara lake spark-template maintenance --config",
        "trellara fleet control-plane --config",
        "trellara pilot-evidence --config",
        "trellara pilot evidence-template --config",
        "trellara pilot evidence-check --config",
        "trellara evidence-registry --package",
    ] {
        assert!(output.contains(marker));
    }
}

fn assert_package_readme(output_path: &std::path::Path) {
    let readme = fs::read_to_string(output_path.join("README.md")).expect("package readme");
    for marker in [
        "trellara check --config",
        "trellara preflight --config",
        "trellara run --local --verify --format text --config",
        "trellara status --config",
        "trellara lake fanin plan --config",
        "trellara lake fanin ddl --config",
        "trellara lake fanin epoch-spec --config",
        "trellara lake fanin run --config",
        "--view report --format text",
    ] {
        assert!(readme.contains(marker));
    }
}

fn assert_package_files(output_path: &std::path::Path) {
    for file in [
        "README.md",
        "pilot-guide.txt",
        "pilot-scorecard.txt",
        "executive-evidence.md",
        "enterprise-evaluation.txt",
        "enterprise-evaluation.json",
        "schema-ddl-plan.json",
        "schema-ddl-apply-plan.json",
        "schema-ddl-envelope-plan.json",
        "ddl-barrier-status.json",
        "ddl-release-proof.json",
        "local-run-proof.md",
        "source-safety-checklist.md",
        "proof-bundle.md",
        "deployment-guide.md",
        "operational-burden-notes.md",
        "feature-pull-list.md",
        "fleet-report.txt",
        "fleet-scorecard.txt",
        "fleet-evidence-plan.txt",
        "live-evidence/README.md",
        "live-evidence/collect.sh",
        "identity-audit.json",
        "lake-verify.json",
        "lake-completeness.json",
        "lake-writer-plan.json",
        "sample-envelope.pb",
        "lake-fanin-run.json",
        "spark-current-state.sql",
        "spark-current-state.py",
        "spark-scd2.sql",
        "spark-scd2.py",
        "spark-maintenance.sql",
        "spark-maintenance.py",
        "spark-completeness-dashboard.sql",
        "spark-completeness-dashboard.py",
        "spark-golden-fixture.json",
        "manifest.json",
    ] {
        assert!(output_path.join(file).exists());
    }
}

fn assert_sample_envelope(output_path: &std::path::Path) {
    let sample_envelope =
        fs::read(output_path.join("sample-envelope.pb")).expect("sample envelope");
    let decoded_envelope =
        TransactionEnvelope::decode_checked(&sample_envelope).expect("decode sample envelope");
    assert_eq!(decoded_envelope.dataset_id, "retail-sales");
    assert_eq!(decoded_envelope.transaction_id, "tx-lake-writer-sample");
    assert_eq!(decoded_envelope.ddl_events.len(), 1);
    assert_eq!(decoded_envelope.ddl_events[0].total_order, 1);
    assert_eq!(decoded_envelope.schema_versions.len(), 1);
    assert_eq!(decoded_envelope.schema_versions[0].version, 67_890);
    assert_eq!(decoded_envelope.changes[0].total_order, 2);
}
