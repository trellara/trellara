use std::path::{Path, PathBuf};

pub(crate) fn repository_root_for(config_path: &Path) -> PathBuf {
    if let Some(root) = find_repository_root(config_path) {
        return root;
    }
    if let Ok(current_dir) = std::env::current_dir() {
        if let Some(root) = find_repository_root(&current_dir) {
            return root;
        }
    }
    if let Some(root) = find_repository_root(Path::new(env!("CARGO_MANIFEST_DIR"))) {
        return root;
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn find_repository_root(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| candidate.join("README.md").exists() && candidate.join("crates").exists())
        .map(Path::to_path_buf)
}
