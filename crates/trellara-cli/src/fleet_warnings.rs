use std::collections::BTreeMap;

use crate::FleetFlowSummary;

pub(crate) fn fleet_warnings(flows: &[FleetFlowSummary]) -> Vec<String> {
    let mut warnings = Vec::new();
    warnings.extend(duplicate_flow_warnings(flows));
    warnings.extend(missing_target_warnings(flows));
    warnings.extend(kafka_readiness_warnings(flows));
    warnings
}

fn duplicate_flow_warnings(flows: &[FleetFlowSummary]) -> Vec<String> {
    let mut flow_ids = BTreeMap::<&str, usize>::new();
    for flow in flows {
        *flow_ids.entry(flow.flow_id.as_str()).or_default() += 1;
    }

    flow_ids
        .into_iter()
        .filter(|(_, count)| *count > 1)
        .map(|(flow_id, count)| {
            format!(
                "duplicate flow_id {flow_id} appears {count} times; control-plane identity would collide"
            )
        })
        .collect()
}

fn missing_target_warnings(flows: &[FleetFlowSummary]) -> Vec<String> {
    let missing_targets = flows
        .iter()
        .filter(|flow| !flow.target_configured)
        .map(|flow| flow.flow_id.clone())
        .collect::<Vec<_>>();

    if missing_targets.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "missing target config for {}; verification and convergence evidence are blocked",
            missing_targets.join(", ")
        )]
    }
}

fn kafka_readiness_warnings(flows: &[FleetFlowSummary]) -> Vec<String> {
    let kafka_flows = flows
        .iter()
        .filter(|flow| flow.stream_kind == "kafka")
        .map(|flow| flow.flow_id.clone())
        .collect::<Vec<_>>();

    if kafka_flows.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "Kafka-backed flows require broker readiness outside the brokerless quickstart: {}",
            kafka_flows.join(", ")
        )]
    }
}
