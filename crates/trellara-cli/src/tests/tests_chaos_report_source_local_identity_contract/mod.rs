use super::*;

mod local_stream;
mod replica_identity;
mod reseed;
mod source_failover;

pub(super) fn assert_chaos_report_source_local_identity_contract(summary: &ChaosRunSummary) {
    source_failover::assert_source_failover_contract(summary);
    local_stream::assert_local_stream_contract(summary);
    reseed::assert_reseed_contract(summary);
    replica_identity::assert_replica_identity_contract(summary);
}

fn scenario<'a>(summary: &'a ChaosRunSummary, name: &str) -> &'a ChaosScenarioSummary {
    summary
        .scenarios
        .iter()
        .find(|scenario| scenario.name == name)
        .unwrap_or_else(|| panic!("missing {name} scenario"))
}
