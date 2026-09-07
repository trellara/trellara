use crate::{DdlPlanVerdict, DdlPropagationPhase};

pub(crate) fn ddl_propagation_phases(verdict: DdlPlanVerdict) -> Vec<DdlPropagationPhase> {
    match verdict {
        DdlPlanVerdict::Blocked => vec![
            phase(
                1,
                "classify_and_block",
                "operator fixes blocked DDL policy, mapping, or destructive-change plan",
            ),
            phase(
                2,
                "fresh_contract_handoff",
                "schema-discover and contract-test pass with the revised config before DML resumes",
            ),
        ],
        DdlPlanVerdict::RequiresManualReview => vec![
            phase(
                1,
                "capture_schema_barrier",
                "later row changes are durably buffered but not visible downstream",
            ),
            phase(
                2,
                "operator_approval",
                "every manual or mapping-required change has named approval and target action",
            ),
            phase(
                3,
                "sink_acknowledgement",
                "all affected sinks acknowledge the accepted schema version and handoff LSN",
            ),
        ],
        DdlPlanVerdict::ReadyToApply => vec![
            phase(
                1,
                "capture_schema_barrier",
                "DDL transaction boundary is recorded before later DML is released",
            ),
            phase(
                2,
                "apply_safe_ddl",
                "compatible DDL is applied or staged on every affected sink",
            ),
            phase(
                3,
                "release_dml",
                "all sink acknowledgements match the barrier schema version and LSN",
            ),
        ],
    }
}

fn phase(order: u32, name: &str, release_condition: &str) -> DdlPropagationPhase {
    DdlPropagationPhase {
        order,
        name: name.to_string(),
        release_condition: release_condition.to_string(),
    }
}
