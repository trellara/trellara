use serde::Serialize;

use crate::{FleetConvergenceGate, FleetLakeFaninReadiness, FleetRecoveryDrill};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetReportSummary {
    pub(crate) flow_count: usize,
    pub(crate) source_count: usize,
    pub(crate) dataset_count: usize,
    pub(crate) table_count: usize,
    pub(crate) target_configured_count: usize,
    pub(crate) local_stream_count: usize,
    pub(crate) kafka_stream_count: usize,
    pub(crate) partitioned_flow_count: usize,
    pub(crate) strict_chunked_flow_count: usize,
    pub(crate) convergence_gate_count: usize,
    pub(crate) blocked_convergence_gate_count: usize,
    pub(crate) recovery_drill_count: usize,
    pub(crate) lake_ready_flow_count: usize,
    pub(crate) lake_publishable_with_gaps_flow_count: usize,
    pub(crate) lake_blocked_flow_count: usize,
    pub(crate) lake_fanin_verdict: String,
    pub(crate) topology_verdict: String,
    pub(crate) flows: Vec<FleetFlowSummary>,
    pub(crate) warnings: Vec<String>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) next_commands: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FleetFlowSummary {
    pub(crate) flow_id: String,
    pub(crate) config: String,
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) mode: String,
    pub(crate) stream_kind: String,
    pub(crate) target_configured: bool,
    pub(crate) table_count: usize,
    pub(crate) tables: Vec<String>,
    pub(crate) topics: Vec<String>,
    pub(crate) transaction_boundary: String,
    pub(crate) lake_fanin: FleetLakeFaninReadiness,
    pub(crate) convergence_gates: Vec<FleetConvergenceGate>,
    pub(crate) recovery_drills: Vec<FleetRecoveryDrill>,
    pub(crate) proof_commands: Vec<String>,
    pub(crate) risks: Vec<String>,
}
