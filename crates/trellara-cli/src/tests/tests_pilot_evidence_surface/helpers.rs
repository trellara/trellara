use super::*;

pub(super) struct PilotEvidencePackageFixture {
    pub(super) root: std::path::PathBuf,
    pub(super) package_path: std::path::PathBuf,
    pub(super) correctness_report_path: std::path::PathBuf,
}

impl PilotEvidencePackageFixture {
    pub(super) fn new(label: &str) -> Self {
        Self::new_with_yaml(label, local_stream_yaml())
    }

    pub(super) fn new_with_yaml(label: &str, yaml: impl AsRef<str>) -> Self {
        let root = std::env::temp_dir().join(format!(
            "trellara-{label}-{}-{}",
            std::process::id(),
            unique_test_suffix()
        ));
        let config_path = root.join("trellara.yml");
        let package_path = root.join("package");
        let correctness_report_path = root.join("correctness-report.html");
        fs::create_dir_all(&root).expect("create evidence registry temp dir");
        fs::write(&config_path, yaml.as_ref()).expect("write evidence registry config");
        fs::write(
            &correctness_report_path,
            "<html><body>Trellara correctness report</body></html>",
        )
        .expect("write correctness report");
        let config = TrellaraConfig::from_path(&config_path).expect("parse config");
        write_pilot_package(
            &config,
            &config_path,
            &package_path,
            &correctness_report_path,
        )
        .expect("write pilot package");

        Self {
            root,
            package_path,
            correctness_report_path,
        }
    }
}

impl Drop for PilotEvidencePackageFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
