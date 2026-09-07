use serde::{Deserialize, Serialize};

use crate::{Result, StreamError};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum StreamMode {
    Strict,
    Manifest,
    Commit,
    Partition { partition: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct TopicLayout {
    source_id: String,
    dataset_id: String,
}

impl TopicLayout {
    pub fn new(source_id: impl Into<String>, dataset_id: impl Into<String>) -> Result<Self> {
        let source_id = source_id.into();
        let dataset_id = dataset_id.into();
        validate_topic_component(&source_id)?;
        validate_topic_component(&dataset_id)?;
        Ok(Self {
            source_id,
            dataset_id,
        })
    }

    pub fn strict_topic(&self) -> String {
        format!("trellara.{}.{}.strict", self.source_id, self.dataset_id)
    }

    pub fn manifest_topic(&self) -> String {
        format!("trellara.{}.{}.manifest", self.source_id, self.dataset_id)
    }

    pub fn commit_topic(&self) -> String {
        format!("trellara.{}.{}.commit", self.source_id, self.dataset_id)
    }

    pub fn partition_topic(&self, partition: u32) -> String {
        format!(
            "trellara.{}.{}.partition.{}",
            self.source_id, self.dataset_id, partition
        )
    }

    pub fn topic_for(&self, mode: StreamMode) -> String {
        match mode {
            StreamMode::Strict => self.strict_topic(),
            StreamMode::Manifest => self.manifest_topic(),
            StreamMode::Commit => self.commit_topic(),
            StreamMode::Partition { partition } => self.partition_topic(partition),
        }
    }
}

fn validate_topic_component(component: &str) -> Result<()> {
    if component.is_empty() {
        return Err(StreamError::InvalidTopicComponent {
            component: component.to_string(),
            reason: "component must not be empty".to_string(),
        });
    }

    if component
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        Ok(())
    } else {
        Err(StreamError::InvalidTopicComponent {
            component: component.to_string(),
            reason: "only ascii letters, numbers, dashes, and underscores are allowed".to_string(),
        })
    }
}
