use std::fs;
use std::path::Path;

pub(crate) fn product_positioning_artifacts_are_current(repository_root: &Path) -> bool {
    fs::read_to_string(repository_root.join("README.md")).is_ok_and(|readme| {
        let readme = readme.to_ascii_lowercase();
        readme.contains("source-safety and verified replication")
            && readme.contains("docs/design.md")
            && readme.contains("docs/roadmap.md")
            && readme.contains("correctness report")
            && readme.contains("postgres")
            && readme.contains("fleet")
    })
}
