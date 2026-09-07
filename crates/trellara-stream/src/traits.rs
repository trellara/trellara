use async_trait::async_trait;

use crate::{Result, StreamMessage};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishAck {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
}

#[async_trait]
pub trait StreamPublisher: Send + Sync {
    async fn publish(&self, message: StreamMessage) -> Result<PublishAck>;
}

#[async_trait]
pub trait StreamConsumer: Send {
    async fn next(&mut self) -> Result<Option<StreamMessage>>;
    async fn ack(&mut self, message: &StreamMessage) -> Result<()>;
}
