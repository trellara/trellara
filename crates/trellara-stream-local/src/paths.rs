use std::path::{Path, PathBuf};

pub(crate) fn topic_dir(root: &Path) -> PathBuf {
    root.join("topics")
}

pub(crate) fn cursor_dir(root: &Path, group_id: &str) -> PathBuf {
    root.join("cursors").join(group_id)
}

pub(crate) fn topic_path(root: &Path, topic: &str) -> PathBuf {
    topic_dir(root).join(format!("{topic}.log"))
}

pub(crate) fn topic_index_path(root: &Path, topic: &str) -> PathBuf {
    topic_dir(root).join(format!("{topic}.idx"))
}

pub(crate) fn cursor_path(root: &Path, group_id: &str, topic: &str) -> PathBuf {
    cursor_dir(root, group_id).join(format!("{topic}.cursor"))
}
