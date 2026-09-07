use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::*;
use async_trait::async_trait;
use trellara_stream::{StreamConsumer, StreamError};

#[derive(Clone)]
pub(in crate::tests) struct RecordingConsumer {
    messages: Arc<Mutex<VecDeque<StreamMessage>>>,
    acked_keys: Arc<Mutex<Vec<String>>>,
    ack_attempts: Arc<Mutex<usize>>,
    fail_ack_attempt: Arc<Mutex<Option<usize>>>,
}

impl RecordingConsumer {
    pub(in crate::tests) fn new(messages: Vec<StreamMessage>) -> Self {
        Self {
            messages: Arc::new(Mutex::new(VecDeque::from(messages))),
            acked_keys: Arc::new(Mutex::new(Vec::new())),
            ack_attempts: Arc::new(Mutex::new(0)),
            fail_ack_attempt: Arc::new(Mutex::new(None)),
        }
    }

    pub(in crate::tests) fn failing_first_ack(messages: Vec<StreamMessage>) -> Self {
        Self::failing_ack_attempt(messages, 1)
    }

    pub(in crate::tests) fn failing_second_ack(messages: Vec<StreamMessage>) -> Self {
        Self::failing_ack_attempt(messages, 2)
    }

    fn failing_ack_attempt(messages: Vec<StreamMessage>, ack_attempt: usize) -> Self {
        let consumer = Self::new(messages);
        *consumer.fail_ack_attempt.lock().expect("fail ack lock") = Some(ack_attempt);
        consumer
    }

    pub(in crate::tests) fn acked_keys(&self) -> Vec<String> {
        self.acked_keys.lock().expect("ack lock").clone()
    }
}

#[async_trait]
impl StreamConsumer for RecordingConsumer {
    async fn next(&mut self) -> trellara_stream::Result<Option<StreamMessage>> {
        Ok(self.messages.lock().expect("message lock").pop_front())
    }

    async fn ack(&mut self, message: &StreamMessage) -> trellara_stream::Result<()> {
        let mut ack_attempts = self.ack_attempts.lock().expect("ack attempts lock");
        *ack_attempts = ack_attempts.saturating_add(1);
        let ack_attempt = *ack_attempts;
        drop(ack_attempts);

        let mut fail_ack_attempt = self.fail_ack_attempt.lock().expect("fail ack lock");
        if *fail_ack_attempt == Some(ack_attempt) {
            *fail_ack_attempt = None;
            return Err(StreamError::Consumer(
                "simulated ack failure after apply".to_string(),
            ));
        }
        drop(fail_ack_attempt);

        self.acked_keys
            .lock()
            .expect("ack lock")
            .push(message.key.clone());
        Ok(())
    }
}

#[derive(Clone)]
pub(in crate::tests) struct RecordingApplier {
    outcomes: Arc<Mutex<VecDeque<Result<ApplyOutcome>>>>,
    applied_transactions: Arc<Mutex<Vec<String>>>,
    applied_manifest_partition_counts: Arc<Mutex<Vec<usize>>>,
}

impl RecordingApplier {
    pub(in crate::tests) fn new(outcomes: Vec<Result<ApplyOutcome>>) -> Self {
        Self {
            outcomes: Arc::new(Mutex::new(VecDeque::from(outcomes))),
            applied_transactions: Arc::new(Mutex::new(Vec::new())),
            applied_manifest_partition_counts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub(in crate::tests) fn applied_transactions(&self) -> Vec<String> {
        self.applied_transactions
            .lock()
            .expect("applied lock")
            .clone()
    }

    pub(in crate::tests) fn applied_manifest_partition_counts(&self) -> Vec<usize> {
        self.applied_manifest_partition_counts
            .lock()
            .expect("manifest lock")
            .clone()
    }
}

#[async_trait]
impl EnvelopeApplier for RecordingApplier {
    async fn apply_envelope(&mut self, envelope: &TransactionEnvelope) -> Result<ApplyOutcome> {
        self.applied_transactions
            .lock()
            .expect("applied lock")
            .push(envelope.transaction_id.clone());
        self.applied_manifest_partition_counts
            .lock()
            .expect("manifest lock")
            .push(
                envelope
                    .manifest
                    .as_ref()
                    .map(|manifest| manifest.partitions.len())
                    .unwrap_or_default(),
            );
        self.outcomes
            .lock()
            .expect("outcome lock")
            .pop_front()
            .expect("recorded apply outcome")
    }
}
