use serde::Serialize;

use crate::StreamConfig;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowStreamSummary {
    pub(crate) kind: String,
    pub(crate) bootstrap_servers: String,
    pub(crate) primary_topic: String,
}

impl FlowStreamSummary {
    pub(crate) fn from_config(stream: &StreamConfig) -> Self {
        match stream {
            StreamConfig::Kafka {
                bootstrap_servers,
                topic,
                ..
            } => Self {
                kind: "kafka".to_string(),
                bootstrap_servers: bootstrap_servers.clone(),
                primary_topic: topic.clone(),
            },
            StreamConfig::Local { path, .. } => Self {
                kind: "local".to_string(),
                bootstrap_servers: path.display().to_string(),
                primary_topic: "local durable segment log".to_string(),
            },
        }
    }
}
