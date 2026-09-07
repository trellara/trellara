use std::collections::{HashMap, HashSet};

use crate::{parse_lsn, DdlBarrier, DdlBarrierAck, DdlBarrierSinkEvidence};

#[derive(Default)]
pub(crate) struct DdlBarrierSummaryComponents {
    pub(crate) acked_sinks: Vec<String>,
    pub(crate) pending_sinks: Vec<String>,
    pub(crate) rejected_sinks: Vec<String>,
    pub(crate) unexpected_sinks: Vec<String>,
    pub(crate) sink_evidence: Vec<DdlBarrierSinkEvidence>,
}

impl DdlBarrierSummaryComponents {
    pub(crate) fn from_barrier_and_acks(barrier: &DdlBarrier, acks: Vec<DdlBarrierAck>) -> Self {
        let ack_by_sink = group_acks_by_sink(acks);
        let required_sinks = barrier.required_sinks.iter().collect::<HashSet<_>>();
        let barrier_lsn = parse_lsn(&barrier.barrier_lsn);
        let mut components = Self::default();

        for sink in &barrier.required_sinks {
            components.add_required_sink(
                sink,
                ack_by_sink.get(sink).map(Vec::as_slice),
                &barrier.schema_version,
                barrier_lsn,
            );
        }

        for (sink, acks) in &ack_by_sink {
            if !required_sinks.contains(sink) {
                components.add_unexpected_sink(sink, acks);
            }
        }

        components.sort();
        components
    }

    fn add_required_sink(
        &mut self,
        sink: &str,
        acks: Option<&[DdlBarrierAck]>,
        schema_version: &str,
        barrier_lsn: u64,
    ) {
        let evidence =
            DdlBarrierSinkEvidence::from_required_sink(sink, acks, schema_version, barrier_lsn);
        match evidence.status.as_str() {
            "acked" => self.acked_sinks.push(sink.to_string()),
            "rejected" => self.rejected_sinks.push(sink.to_string()),
            _ => self.pending_sinks.push(sink.to_string()),
        }
        self.sink_evidence.push(evidence);
    }

    fn add_unexpected_sink(&mut self, sink: &str, acks: &[DdlBarrierAck]) {
        self.unexpected_sinks.push(sink.to_string());
        self.sink_evidence
            .push(DdlBarrierSinkEvidence::from_unexpected_acks(sink, acks));
    }

    fn sort(&mut self) {
        self.acked_sinks.sort();
        self.pending_sinks.sort();
        self.rejected_sinks.sort();
        self.unexpected_sinks.sort();
        self.sink_evidence
            .sort_by(|left, right| left.sink.cmp(&right.sink));
    }
}

fn group_acks_by_sink(acks: Vec<DdlBarrierAck>) -> HashMap<String, Vec<DdlBarrierAck>> {
    acks.into_iter().fold(
        HashMap::<String, Vec<DdlBarrierAck>>::new(),
        |mut map, ack| {
            map.entry(ack.sink.clone()).or_default().push(ack);
            map
        },
    )
}
