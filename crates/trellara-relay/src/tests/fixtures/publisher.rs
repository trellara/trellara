use std::sync::{Arc, Mutex};

use super::*;
use async_trait::async_trait;
use trellara_stream::{StreamMessage, StreamPublisher};

#[derive(Clone)]
pub(in crate::tests) struct RecordingPublisher {
    published: Arc<Mutex<Vec<StreamMessage>>>,
    fail: bool,
    fail_after_record: bool,
    fail_on_attempt: Option<usize>,
    fail_after_record_on_attempt: Option<usize>,
}

impl RecordingPublisher {
    pub(in crate::tests) fn succeeding() -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
            fail: false,
            fail_after_record: false,
            fail_on_attempt: None,
            fail_after_record_on_attempt: None,
        }
    }

    pub(in crate::tests) fn failing() -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
            fail: true,
            fail_after_record: false,
            fail_on_attempt: None,
            fail_after_record_on_attempt: None,
        }
    }

    pub(in crate::tests) fn ambiguous_after_broker_accept() -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
            fail: false,
            fail_after_record: true,
            fail_on_attempt: None,
            fail_after_record_on_attempt: None,
        }
    }

    pub(in crate::tests) fn failing_on_attempt(attempt: usize) -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
            fail: false,
            fail_after_record: false,
            fail_on_attempt: Some(attempt),
            fail_after_record_on_attempt: None,
        }
    }

    pub(in crate::tests) fn ambiguous_on_attempt(attempt: usize) -> Self {
        Self {
            published: Arc::new(Mutex::new(Vec::new())),
            fail: false,
            fail_after_record: false,
            fail_on_attempt: None,
            fail_after_record_on_attempt: Some(attempt),
        }
    }

    pub(in crate::tests) fn published_messages(&self) -> Vec<StreamMessage> {
        self.published.lock().expect("publisher lock").clone()
    }
}

#[async_trait]
impl StreamPublisher for RecordingPublisher {
    async fn publish(&self, message: StreamMessage) -> trellara_stream::Result<PublishAck> {
        if self.fail {
            return Err(StreamError::Publisher(
                "simulated publish failure".to_string(),
            ));
        }

        let mut published = self.published.lock().expect("publisher lock");
        let attempt = published.len() + 1;
        if self.fail_on_attempt == Some(attempt) {
            return Err(StreamError::Publisher(format!(
                "simulated publish failure on attempt {attempt}"
            )));
        }
        let offset = published.len() as i64;
        let topic = message.topic.clone();
        let partition = message.partition.unwrap_or(0);
        published.push(message);
        if self.fail_after_record || self.fail_after_record_on_attempt == Some(attempt) {
            return Err(StreamError::Publisher(format!(
                "simulated ambiguous publish acknowledgement on attempt {attempt}"
            )));
        }
        Ok(PublishAck {
            topic,
            partition,
            offset,
        })
    }
}
