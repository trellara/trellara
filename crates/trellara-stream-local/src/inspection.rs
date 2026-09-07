use std::path::Path;

use crate::inspection_cursors::inspect_cursors;
use crate::inspection_topics::inspect_topics;
use crate::{LocalStreamInspection, Result};

pub fn inspect_local_stream(root: impl AsRef<Path>) -> Result<LocalStreamInspection> {
    let root = root.as_ref().to_path_buf();
    let topics = inspect_topics(&root)?;
    let message_counts = topics
        .iter()
        .map(|topic| (topic.topic.clone(), topic.message_count))
        .collect();
    Ok(LocalStreamInspection {
        cursors: inspect_cursors(&root, &message_counts)?,
        topics,
        root,
    })
}
