use std::sync::{Arc, Mutex};

use rdkafka::client::ClientContext;
use rdkafka::consumer::{BaseConsumer, CommitMode, Consumer, ConsumerContext, Rebalance};

use crate::consumer_progress::ConsumerProgress;

#[derive(Default)]
pub(crate) struct KafkaConsumerState {
    pub(crate) progress: Mutex<ConsumerProgress>,
    callback_error: Mutex<Option<String>>,
}

impl KafkaConsumerState {
    pub(crate) fn take_callback_error(&self) -> Option<String> {
        self.callback_error
            .lock()
            .ok()
            .and_then(|mut error| error.take())
    }
}

#[derive(Clone, Default)]
pub(crate) struct KafkaConsumerContext {
    pub(crate) state: Arc<KafkaConsumerState>,
}

impl ClientContext for KafkaConsumerContext {}

impl ConsumerContext for KafkaConsumerContext {
    fn pre_rebalance(&self, consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        match rebalance {
            Rebalance::Revoke(partitions) => self.revoke(consumer, partitions),
            Rebalance::Error(error) => {
                self.record_callback_error(format!("consumer rebalance failed: {error}"));
            }
            Rebalance::Assign(_) => {}
        }
    }

    fn post_rebalance(&self, _consumer: &BaseConsumer<Self>, rebalance: &Rebalance<'_>) {
        if let Rebalance::Assign(partitions) = rebalance {
            if let Ok(mut progress) = self.state.progress.lock() {
                progress.assign(partitions);
            }
        }
    }
}

impl KafkaConsumerContext {
    fn revoke(&self, consumer: &BaseConsumer<Self>, partitions: &rdkafka::TopicPartitionList) {
        if consumer.assignment_lost() {
            self.record_callback_error(
                "consumer assignment was lost; revoked offsets were not committed and will be redelivered"
                    .to_string(),
            );
            self.drop_revoked_progress(partitions);
            return;
        }
        self.commit_revoked_progress(consumer, partitions);
        self.drop_revoked_progress(partitions);
    }

    fn commit_revoked_progress(
        &self,
        consumer: &BaseConsumer<Self>,
        partitions: &rdkafka::TopicPartitionList,
    ) {
        let pending = self
            .state
            .progress
            .lock()
            .ok()
            .and_then(|progress| progress.pending_commit_for(partitions));
        if let Some(pending) = pending {
            match consumer.commit(&pending.offsets, CommitMode::Sync) {
                Ok(()) => {
                    if let Ok(mut progress) = self.state.progress.lock() {
                        progress.commit_succeeded(pending);
                    }
                }
                Err(error) => self.record_callback_error(format!(
                    "synchronous commit during partition revoke failed: {error}"
                )),
            }
        }
    }

    fn drop_revoked_progress(&self, partitions: &rdkafka::TopicPartitionList) {
        if let Ok(mut progress) = self.state.progress.lock() {
            progress.revoke(partitions);
        }
    }

    fn record_callback_error(&self, error: String) {
        if let Ok(mut callback_error) = self.state.callback_error.lock() {
            *callback_error = Some(error);
        }
    }
}
