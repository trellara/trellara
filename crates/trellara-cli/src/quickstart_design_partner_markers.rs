pub(crate) use crate::quickstart_design_partner_makefile_markers::{
    MAKEFILE_JSON_FRAGMENTS, MAKEFILE_MARKERS,
};
pub(crate) use crate::quickstart_design_partner_readme_markers::README_MARKERS;

pub(crate) fn all_markers_present(content: &str, markers: &[&str]) -> bool {
    markers.iter().all(|marker| content.contains(marker))
}

pub(crate) fn all_json_fragments_present(content: &str, fragments: &[&str]) -> bool {
    fragments
        .iter()
        .all(|fragment| contains_json_fragment(content, fragment))
}

pub(crate) fn pilot_package_artifact_count_present(content: &str) -> bool {
    contains_json_fragment(content, "\"artifact_count\": 51")
}

fn contains_json_fragment(content: &str, fragment: &str) -> bool {
    content.contains(fragment) || content.contains(&fragment.replace('"', "\\\""))
}
