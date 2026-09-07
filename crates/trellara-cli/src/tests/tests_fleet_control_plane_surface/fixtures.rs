use std::path::PathBuf;

use super::*;

pub(super) struct FleetControlPlaneFixture {
    root: PathBuf,
    east_path: PathBuf,
    west_path: PathBuf,
}

impl FleetControlPlaneFixture {
    pub(super) fn configs(&self) -> Vec<PathBuf> {
        vec![self.east_path.clone(), self.west_path.clone()]
    }

    pub(super) fn remove(self) {
        fs::remove_dir_all(self.root).expect("remove fleet control-plane temp dir");
    }
}

pub(super) fn two_flow_control_plane_fixture(prefix: &str) -> FleetControlPlaneFixture {
    let root = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    fs::create_dir_all(&root).expect("create fleet control-plane temp dir");
    let east_path = root.join("east.yml");
    let west_path = root.join("west.yml");
    fs::write(
        &east_path,
        local_stream_yaml()
            .replace("id: local-source", "id: east-source")
            .replace("id: retail-sales", "id: retail-east"),
    )
    .expect("write east config");
    fs::write(
        &west_path,
        local_stream_yaml()
            .replace("id: local-source", "id: west-source")
            .replace("id: retail-sales", "id: retail-west"),
    )
    .expect("write west config");
    FleetControlPlaneFixture {
        root,
        east_path,
        west_path,
    }
}
