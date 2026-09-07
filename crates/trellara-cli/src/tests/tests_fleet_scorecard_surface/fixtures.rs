use std::path::PathBuf;

use super::*;

pub(super) struct FleetScorecardFixture {
    root: PathBuf,
    configs: Vec<PathBuf>,
    partitioned_path: Option<PathBuf>,
}

impl FleetScorecardFixture {
    pub(super) fn configs(&self) -> Vec<PathBuf> {
        self.configs.clone()
    }

    pub(super) fn partitioned_display(&self) -> String {
        self.partitioned_path
            .as_ref()
            .expect("partitioned fixture path")
            .display()
            .to_string()
    }

    pub(super) fn remove(self) {
        fs::remove_dir_all(self.root).expect("remove fleet scorecard temp dir");
    }
}

pub(super) fn mixed_fleet_scorecard_fixture() -> FleetScorecardFixture {
    let root = temp_root("trellara-fleet-scorecard");
    let local_path = root.join("local.yml");
    let partitioned_path = root.join("partitioned.yml");
    let strict_chunked_local_yaml = local_stream_yaml().replace(
        "  tables:\n    - schema: public\n      name: sales",
        "  tables:\n    - schema: public\n      name: sales\n  strict_chunking:\n    max_changes_per_chunk: 1000",
    );
    fs::write(&local_path, strict_chunked_local_yaml).expect("write local config");
    fs::write(&partitioned_path, partitioned_yaml()).expect("write partitioned config");
    FleetScorecardFixture {
        root,
        configs: vec![local_path, partitioned_path.clone()],
        partitioned_path: Some(partitioned_path),
    }
}

pub(super) fn source_only_scorecard_fixture() -> FleetScorecardFixture {
    let root = temp_root("trellara-fleet-scorecard-missing-target");
    let config_path = root.join("source-only.yml");
    let yaml = local_stream_yaml()
        .replace("\ntarget:\n  database_url: postgresql://target/app\n", "\n")
        .replace(
            "\ntarget:\n  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target\n",
            "\n",
        );
    fs::write(&config_path, yaml).expect("write source-only config");
    FleetScorecardFixture {
        root,
        configs: vec![config_path],
        partitioned_path: None,
    }
}

fn temp_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet scorecard temp dir");
    root
}
