use std::path::Path;

#[path = "quickstart_schema_artifacts/cli.rs"]
mod cli;
#[path = "quickstart_schema_artifacts/makefile.rs"]
mod makefile;
#[path = "quickstart_schema_artifacts/readme.rs"]
mod readme;

pub(crate) fn schema_ddl_propagation_artifacts_are_current(repository_root: &Path) -> bool {
    readme::is_current(repository_root)
        && makefile::is_current(repository_root)
        && cli::is_current(repository_root)
}
