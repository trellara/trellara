use crate::fleet_fanin::FleetFanInSimulationConfig;
use crate::fleet_fanin_fixture::fleet_envelope;
use trellara_protocol::TransactionEnvelope;

pub(crate) struct FleetFanInWorkload {
    pub(crate) sources: Vec<String>,
    pub(crate) online_envelopes: Vec<TransactionEnvelope>,
    pub(crate) late_envelopes: Vec<TransactionEnvelope>,
}

impl FleetFanInWorkload {
    pub(crate) fn new(config: &FleetFanInSimulationConfig) -> Self {
        let store_count = config.store_count.max(config.offline_store_count);
        let sources = source_ids(store_count);
        let online_count = store_count.saturating_sub(config.offline_store_count);
        let online_envelopes = source_envelopes(config, sources.iter().take(online_count), 1);
        let late_envelopes =
            source_envelopes(config, sources.iter().skip(online_count), online_count + 1);

        Self {
            sources,
            online_envelopes,
            late_envelopes,
        }
    }

    pub(crate) fn missing_sources(&self) -> impl Iterator<Item = String> + '_ {
        self.sources
            .iter()
            .skip(self.online_envelopes.len())
            .cloned()
    }
}

fn source_ids(store_count: usize) -> Vec<String> {
    (0..store_count)
        .map(|index| format!("store-{:04}", index + 1))
        .collect()
}

fn source_envelopes<'a>(
    config: &FleetFanInSimulationConfig,
    source_ids: impl Iterator<Item = &'a String>,
    first_ordinal: usize,
) -> Vec<TransactionEnvelope> {
    source_ids
        .enumerate()
        .map(|(index, source_id)| {
            fleet_envelope(
                config.seed,
                &config.dataset_id,
                source_id,
                first_ordinal + index,
                false,
            )
        })
        .collect()
}
