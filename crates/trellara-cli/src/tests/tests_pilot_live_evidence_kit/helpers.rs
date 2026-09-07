use super::*;

pub(super) fn temp_root(label: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "trellara-{label}-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ))
}

pub(super) fn write_local_config(root: &std::path::Path) -> std::path::PathBuf {
    let config_path = root.join("trellara.yml");
    fs::write(&config_path, local_stream_yaml()).expect("write config");
    config_path
}

pub(super) fn write_partitioned_config(root: &std::path::Path) -> std::path::PathBuf {
    let config_path = root.join("partitioned.yml");
    fs::write(&config_path, local_partitioned_yaml()).expect("write partitioned config");
    config_path
}
