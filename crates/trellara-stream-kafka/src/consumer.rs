use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer};
use rdkafka::util::Timeout;
use trellara_stream::{StreamConsumer, StreamError, StreamMessage};

use crate::consumer_config::KafkaConsumerConfig;
use crate::consumer_context::{KafkaConsumerContext, KafkaConsumerState};
use crate::kafka_message::stream_message_from_kafka;
use crate::topology_snapshot::KafkaTopologySnapshot;
use crate::KafkaStreamError;

pub struct KafkaConsumer {
    consumer: BaseConsumer<KafkaConsumerContext>,
    state: Arc<KafkaConsumerState>,
    poll_timeout: Duration,
    max_in_flight_messages: usize,
    paused_for_backpressure: bool,
}

impl KafkaConsumer {
    pub fn new(config: KafkaConsumerConfig) -> Result<Self, KafkaStreamError> {
        config.validate()?;
        let mut client_config = ClientConfig::new();
        client_config
            .set("bootstrap.servers", &config.bootstrap_servers)
            .set("group.id", &config.group_id)
            .set("client.id", &config.client_id)
            .set("enable.auto.commit", "false")
            .set("enable.auto.offset.store", "false")
            .set("auto.offset.reset", "earliest")
            .set("partition.assignment.strategy", "cooperative-sticky")
            .set(
                "max.poll.interval.ms",
                config.max_poll_interval.as_millis().to_string(),
            )
            .set(
                "queued.max.messages.kbytes",
                config.queued_max_messages_kbytes.to_string(),
            );
        if let Some(security) = &config.security {
            security.apply(&mut client_config)?;
        }

        let context = KafkaConsumerContext::default();
        let state = Arc::clone(&context.state);
        let consumer: BaseConsumer<KafkaConsumerContext> = client_config
            .create_with_context(context)
            .map_err(|_| KafkaStreamError::ClientInitialization { client: "consumer" })?;
        if let Some(requirement) = &config.topology {
            let metadata =
                consumer.fetch_metadata(None, Timeout::After(requirement.metadata_timeout))?;
            KafkaTopologySnapshot::from_metadata(&metadata, &config.topics)
                .validate(&config.topics, requirement)?;
        }
        let topics = config.topics.iter().map(String::as_str).collect::<Vec<_>>();
        consumer.subscribe(&topics)?;

        Ok(Self {
            consumer,
            state,
            poll_timeout: config.poll_timeout,
            max_in_flight_messages: config.max_in_flight_messages,
            paused_for_backpressure: false,
        })
    }

    #[must_use]
    pub fn in_flight_messages(&self) -> usize {
        self.state
            .progress
            .lock()
            .map_or(self.max_in_flight_messages, |progress| progress.in_flight())
    }

    #[must_use]
    pub fn is_backpressured(&self) -> bool {
        self.in_flight_messages() >= self.max_in_flight_messages
    }

    fn take_callback_error(&self) -> Option<String> {
        self.state.take_callback_error()
    }

    fn pause_for_backpressure(&mut self) -> Result<(), StreamError> {
        if !self.paused_for_backpressure {
            let assignment = self.consumer.assignment().map_err(consumer_error)?;
            self.consumer.pause(&assignment).map_err(consumer_error)?;
            self.paused_for_backpressure = true;
        }
        Ok(())
    }

    fn resume_after_backpressure(&mut self) -> Result<(), StreamError> {
        if self.paused_for_backpressure && !self.is_backpressured() {
            let assignment = self.consumer.assignment().map_err(consumer_error)?;
            self.consumer.resume(&assignment).map_err(consumer_error)?;
            self.paused_for_backpressure = false;
        }
        Ok(())
    }
}

#[async_trait]
impl StreamConsumer for KafkaConsumer {
    async fn next(&mut self) -> Result<Option<StreamMessage>, StreamError> {
        if let Some(error) = self.take_callback_error() {
            return Err(StreamError::Consumer(error));
        }
        if self.is_backpressured() {
            self.pause_for_backpressure()?;
            std::thread::sleep(self.poll_timeout);
            return Ok(None);
        }
        self.resume_after_backpressure()?;
        match self.consumer.poll(self.poll_timeout) {
            Some(Ok(message)) => {
                let stream_message = stream_message_from_kafka(&message);
                let position = stream_message.position.as_ref().ok_or_else(|| {
                    StreamError::Consumer(
                        "Kafka record did not contain a stream position".to_string(),
                    )
                })?;
                self.state
                    .progress
                    .lock()
                    .map_err(|_| poisoned_progress())?
                    .record_delivery(position)
                    .map_err(consumer_error)?;
                Ok(Some(stream_message))
            }
            Some(Err(error)) => Err(consumer_error(error)),
            None => Ok(None),
        }
    }

    async fn ack(&mut self, message: &StreamMessage) -> Result<(), StreamError> {
        if let Some(error) = self.take_callback_error() {
            return Err(StreamError::Consumer(error));
        }
        let position = message.position.as_ref().ok_or_else(|| {
            StreamError::Consumer("cannot ack message without stream position".to_string())
        })?;
        let pending = self
            .state
            .progress
            .lock()
            .map_err(|_| poisoned_progress())?
            .acknowledge(position)
            .map_err(consumer_error)?;
        if let Some(pending) = pending {
            self.consumer
                .commit(&pending.offsets, CommitMode::Sync)
                .map_err(consumer_error)?;
            self.state
                .progress
                .lock()
                .map_err(|_| poisoned_progress())?
                .commit_succeeded(pending);
        }
        self.resume_after_backpressure()
    }
}

fn consumer_error(error: impl std::fmt::Display) -> StreamError {
    StreamError::Consumer(error.to_string())
}

fn poisoned_progress() -> StreamError {
    StreamError::Consumer("Kafka consumer progress state is unavailable".to_string())
}
