use super::*;

pub(super) struct QuickstartConfigFixture {
    pub(super) config_path: PathBuf,
    root: PathBuf,
}

impl Drop for QuickstartConfigFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(super) fn quickstart_config(prefix: &str, yaml: impl AsRef<str>) -> QuickstartConfigFixture {
    let root = std::env::temp_dir().join(format!("{prefix}-{}", unique_test_suffix()));
    let config_path = root.join("trellara.yml");
    fs::create_dir_all(&root).expect("create quickstart temp dir");
    fs::write(&config_path, yaml.as_ref()).expect("write quickstart config");

    QuickstartConfigFixture { config_path, root }
}

pub(super) fn readiness_summary(config_path: PathBuf) -> QuickstartReadinessSummary {
    quickstart_readiness(&QuickstartArgs {
        config: config_path,
        check: true,
        format: QuickstartOutputFormat::Json,
    })
    .expect("quickstart readiness")
}
