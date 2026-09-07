use std::collections::HashMap;
use std::path::PathBuf;

use crate::topic::validate_topic;
use crate::{LocalStreamError, Result};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub enum LocalDurability {
    #[default]
    Fsync,
    Buffered,
}

impl LocalDurability {
    pub(crate) fn sync_enabled(self) -> bool {
        matches!(self, Self::Fsync)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fsync => "fsync",
            Self::Buffered => "buffered",
        }
    }

    pub fn crash_safe_ack(self) -> bool {
        matches!(self, Self::Fsync)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalPublisherConfig {
    pub root: PathBuf,
    pub durability: LocalDurability,
}

impl LocalPublisherConfig {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            durability: LocalDurability::default(),
        }
    }

    pub fn with_durability(mut self, durability: LocalDurability) -> Self {
        self.durability = durability;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LocalConsumerConfig {
    pub root: PathBuf,
    pub group_id: String,
    pub topics: Vec<String>,
    pub durability: LocalDurability,
}

impl LocalConsumerConfig {
    pub fn new(root: impl Into<PathBuf>, group_id: impl Into<String>, topics: Vec<String>) -> Self {
        Self {
            root: root.into(),
            group_id: group_id.into(),
            topics,
            durability: LocalDurability::default(),
        }
    }

    pub fn with_durability(mut self, durability: LocalDurability) -> Self {
        self.durability = durability;
        self
    }

    pub(crate) fn validate(&self) -> Result<()> {
        if self.topics.is_empty() {
            return Err(LocalStreamError::MissingTopics);
        }
        for topic in &self.topics {
            validate_topic(topic)?;
        }
        validate_topic(&self.group_id)
    }
}

#[derive(Clone, Debug)]
pub struct LocalPublisher {
    pub(crate) config: LocalPublisherConfig,
}

#[derive(Clone, Debug)]
pub struct LocalConsumer {
    pub(crate) config: LocalConsumerConfig,
    pub(crate) read_cursors: HashMap<String, LocalReadCursor>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct LocalReadCursor {
    pub(crate) durable_offset: i64,
    pub(crate) next_offset: i64,
}
