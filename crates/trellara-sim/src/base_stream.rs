use std::collections::VecDeque;

use crate::transaction::Transaction;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct BaseStream {
    queue: VecDeque<Transaction>,
    published_messages: usize,
    acknowledged_messages: usize,
}

impl BaseStream {
    pub(crate) fn publish(&mut self, transaction: Transaction) {
        self.queue.push_back(transaction);
        self.published_messages += 1;
    }

    pub(crate) fn redeliver_front(&mut self, transaction: Transaction) {
        self.queue.push_front(transaction);
    }

    pub(crate) fn redeliver_back(&mut self, transaction: Transaction) {
        self.queue.push_back(transaction);
    }

    pub(crate) fn pop_next(&mut self) -> Option<Transaction> {
        self.queue.pop_front()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(crate) fn acknowledge(&mut self) {
        self.acknowledged_messages += 1;
    }

    pub(crate) fn published_messages(&self) -> usize {
        self.published_messages
    }

    pub(crate) fn acknowledged_messages(&self) -> usize {
        self.acknowledged_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tracks_publish_and_ack_counts() {
        let mut stream = BaseStream::default();

        stream.publish(transaction("tx-1", 10));
        stream.publish(transaction("tx-2", 11));
        stream.acknowledge();

        assert_eq!(stream.published_messages(), 2);
        assert_eq!(stream.acknowledged_messages(), 1);
    }

    #[test]
    fn redelivery_front_retries_before_later_messages() {
        let mut stream = BaseStream::default();

        stream.publish(transaction("tx-1", 10));
        stream.publish(transaction("tx-2", 11));
        let first = stream.pop_next().expect("first message");
        stream.redeliver_front(first);

        assert_eq!(stream.pop_next().expect("retried message").id, "tx-1");
        assert_eq!(stream.pop_next().expect("later message").id, "tx-2");
    }

    fn transaction(id: &str, lsn: u64) -> Transaction {
        Transaction {
            id: id.to_string(),
            lsn,
        }
    }
}
