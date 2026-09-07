use std::fs;
use std::path::Path;

pub(super) fn is_current(repository_root: &Path) -> bool {
    fs::read_to_string(repository_root.join("docs").join("DESIGN.md")).is_ok_and(|readme| {
        readme.contains("schema-barrier transaction boundary")
            && readme.contains("structured propagation contract")
            && readme.contains("per-sink acknowledgements")
            && readme.contains("partitioned global-visibility pause")
            && readme.contains("targets accept the new schema version")
    })
}
