use std::cmp::Ordering;

use crate::ddl_barrier_ack_rejection::ddl_ack_rejection;
use crate::{parse_lsn, DdlBarrierAck, DdlBarrierSinkEvidence};

impl DdlBarrierSinkEvidence {
    pub(crate) fn from_required_sink(
        sink: &str,
        acks: Option<&[DdlBarrierAck]>,
        required_schema_version: &str,
        barrier_lsn: u64,
    ) -> Self {
        match acks {
            Some([]) | None => Self::pending(sink),
            Some([ack]) => Self::from_ack(sink, ack, required_schema_version, barrier_lsn),
            Some(acks) => Self::from_duplicate_required_acks(sink, acks),
        }
    }

    pub(crate) fn from_unexpected_acks(sink: &str, acks: &[DdlBarrierAck]) -> Self {
        match representative_ack(acks) {
            Some(ack) => Self::from_unexpected_ack(sink, ack),
            None => Self::unexpected_without_ack(sink),
        }
    }

    fn from_unexpected_ack(sink: &str, ack: &DdlBarrierAck) -> Self {
        Self {
            sink: sink.to_string(),
            status: "unexpected".to_string(),
            ack_lsn: Some(ack.ack_lsn.clone()),
            schema_version: Some(ack.schema_version.clone()),
            accepted: Some(ack.accepted),
            detail: Some(ack.detail.clone()),
            release_eligible: false,
            rejection_code: Some("unexpected_sink_ack".to_string()),
            rejection_reason: Some("sink is not required for this barrier".to_string()),
        }
    }

    fn pending(sink: &str) -> Self {
        Self {
            sink: sink.to_string(),
            status: "pending".to_string(),
            ack_lsn: None,
            schema_version: None,
            accepted: None,
            detail: None,
            release_eligible: false,
            rejection_code: None,
            rejection_reason: None,
        }
    }

    fn from_duplicate_required_acks(sink: &str, acks: &[DdlBarrierAck]) -> Self {
        let ack = representative_ack(acks);
        Self {
            sink: sink.to_string(),
            status: "rejected".to_string(),
            ack_lsn: ack.map(|ack| ack.ack_lsn.clone()),
            schema_version: ack.map(|ack| ack.schema_version.clone()),
            accepted: ack.map(|ack| ack.accepted),
            detail: Some(format!(
                "{} acknowledgements recorded for sink {sink}",
                acks.len()
            )),
            release_eligible: false,
            rejection_code: Some("duplicate_required_ack".to_string()),
            rejection_reason: Some(
                "multiple acknowledgements for required sink must be reconciled".to_string(),
            ),
        }
    }

    fn unexpected_without_ack(sink: &str) -> Self {
        Self {
            sink: sink.to_string(),
            status: "unexpected".to_string(),
            ack_lsn: None,
            schema_version: None,
            accepted: None,
            detail: None,
            release_eligible: false,
            rejection_code: Some("unexpected_sink_ack".to_string()),
            rejection_reason: Some(format!("sink {sink} is not required for this barrier")),
        }
    }

    fn from_ack(
        sink: &str,
        ack: &DdlBarrierAck,
        required_schema_version: &str,
        barrier_lsn: u64,
    ) -> Self {
        let rejection = ddl_ack_rejection(ack, required_schema_version, barrier_lsn);
        let release_eligible = rejection.is_none();
        Self {
            sink: sink.to_string(),
            status: if release_eligible {
                "acked".to_string()
            } else {
                "rejected".to_string()
            },
            ack_lsn: Some(ack.ack_lsn.clone()),
            schema_version: Some(ack.schema_version.clone()),
            accepted: Some(ack.accepted),
            detail: Some(ack.detail.clone()),
            release_eligible,
            rejection_code: rejection
                .as_ref()
                .map(|rejection| rejection.code.to_string()),
            rejection_reason: rejection.map(|rejection| rejection.reason),
        }
    }
}

fn representative_ack(acks: &[DdlBarrierAck]) -> Option<&DdlBarrierAck> {
    acks.iter().max_by(|left, right| compare_ack(left, right))
}

fn compare_ack(left: &DdlBarrierAck, right: &DdlBarrierAck) -> Ordering {
    parse_lsn(&left.ack_lsn)
        .cmp(&parse_lsn(&right.ack_lsn))
        .then_with(|| left.schema_version.cmp(&right.schema_version))
        .then_with(|| left.accepted.cmp(&right.accepted))
        .then_with(|| left.detail.cmp(&right.detail))
        .then_with(|| left.ack_lsn.cmp(&right.ack_lsn))
}
