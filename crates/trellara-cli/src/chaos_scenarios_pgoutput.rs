#[path = "chaos_scenarios_pgoutput/identity.rs"]
mod identity;
#[path = "chaos_scenarios_pgoutput/schema.rs"]
mod schema;
#[path = "chaos_scenarios_pgoutput/transaction.rs"]
mod transaction;

use crate::ChaosScenarioSummary;

pub(crate) fn pgoutput_capture_scenarios() -> Vec<ChaosScenarioSummary> {
    let mut scenarios = Vec::new();
    scenarios.extend(schema::pgoutput_schema_scenarios());
    scenarios.extend(transaction::pgoutput_transaction_scenarios());
    scenarios.extend(identity::pgoutput_identity_scenarios());
    scenarios
}
