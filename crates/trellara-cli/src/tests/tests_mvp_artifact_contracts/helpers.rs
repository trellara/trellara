use super::*;

pub(super) fn temp_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "trellara-{label}-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ))
}
