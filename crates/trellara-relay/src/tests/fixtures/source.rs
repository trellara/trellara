use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use super::*;
use async_trait::async_trait;
use trellara_protocol::TransactionEnvelope;

pub(in crate::tests) struct FakeSource {
    envelopes: VecDeque<TransactionEnvelope>,
    acked_lsns: Arc<Mutex<Vec<String>>>,
    fail_ack: bool,
}

impl FakeSource {
    pub(in crate::tests) fn new(envelopes: Vec<TransactionEnvelope>) -> Self {
        Self {
            envelopes: VecDeque::from(envelopes),
            acked_lsns: Arc::new(Mutex::new(Vec::new())),
            fail_ack: false,
        }
    }

    pub(in crate::tests) fn with_ack_failure(envelopes: Vec<TransactionEnvelope>) -> Self {
        Self {
            envelopes: VecDeque::from(envelopes),
            acked_lsns: Arc::new(Mutex::new(Vec::new())),
            fail_ack: true,
        }
    }

    pub(in crate::tests) fn acked_lsns(&self) -> Arc<Mutex<Vec<String>>> {
        self.acked_lsns.clone()
    }
}

#[async_trait]
impl ChangeSource for FakeSource {
    async fn next_transaction(
        &mut self,
    ) -> trellara_pg_capture::Result<Option<TransactionEnvelope>> {
        Ok(self.envelopes.pop_front())
    }

    async fn acknowledge_durable_lsn(&mut self, lsn: &str) -> trellara_pg_capture::Result<()> {
        if self.fail_ack {
            return Err(CaptureError::ReplicationProtocol(
                "simulated source acknowledgement failure".to_string(),
            ));
        }
        self.acked_lsns
            .lock()
            .expect("acked lsn lock")
            .push(lsn.to_string());
        Ok(())
    }
}
