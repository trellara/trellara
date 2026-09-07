use super::*;

pub(super) struct FleetEvidenceFixture {
    root: PathBuf,
}

impl FleetEvidenceFixture {
    pub(super) fn new(prefix: &str) -> Self {
        let root = std::env::temp_dir().join(format!("{prefix}-{}", unique_test_suffix()));
        fs::create_dir_all(&root).expect("create fleet evidence temp dir");
        Self { root }
    }

    pub(super) fn write_config(&self, file_name: &str, yaml: impl AsRef<str>) -> PathBuf {
        let path = self.root.join(file_name);
        fs::write(&path, yaml.as_ref()).expect("write fleet evidence config");
        path
    }
}

impl Drop for FleetEvidenceFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
