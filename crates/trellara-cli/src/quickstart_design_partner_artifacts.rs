use std::fs;
use std::path::Path;

use crate::{
    all_json_fragments_present, all_markers_present, pilot_package_artifact_count_present,
    MAKEFILE_JSON_FRAGMENTS, MAKEFILE_MARKERS, README_MARKERS,
};

pub(crate) fn local_design_partner_artifacts_are_current(repository_root: &Path) -> bool {
    let readme_current = fs::read_to_string(repository_root.join("README.md"))
        .is_ok_and(|readme| all_markers_present(&readme, README_MARKERS));
    let makefile_current =
        fs::read_to_string(repository_root.join("Makefile")).is_ok_and(|makefile| {
            all_markers_present(&makefile, MAKEFILE_MARKERS)
                && all_json_fragments_present(&makefile, MAKEFILE_JSON_FRAGMENTS)
                && pilot_package_artifact_count_present(&makefile)
        });

    readme_current && makefile_current
}
