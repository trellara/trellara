use super::tests_pilot_package_artifact_contract::assert_pilot_package_artifacts;
use super::tests_pilot_package_command_contract::assert_pilot_package_next_commands;
use super::tests_pilot_package_content_contract::PilotPackageContents;
use super::*;

#[test]
fn pilot_package_writes_design_partner_artifacts() {
    let root = std::env::temp_dir().join(format!("trellara-pilot-package-{}", std::process::id()));
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
    let config = TrellaraConfig::from_path(&config_path).expect("parse config");

    let summary = write_pilot_package(
        &config,
        &config_path,
        &output_path,
        &correctness_report_path,
    )
    .expect("write package");

    assert_pilot_package_artifacts(&summary);
    assert_pilot_package_next_commands(&summary, &config_path, &output_path);

    let contents = PilotPackageContents::read_from(&output_path);
    contents.assert_contract(&config_path);

    fs::remove_dir_all(root).expect("remove pilot package temp dir");
}
