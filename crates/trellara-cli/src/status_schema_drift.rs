use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct FlowSchemaDriftSummary {
    pub(crate) relation_count: usize,
    pub(crate) relations: Vec<String>,
    pub(crate) reason: String,
    pub(crate) recommendation: String,
}

impl FlowSchemaDriftSummary {
    pub(crate) fn from_preflight(tables: &[trellara_pg_capture::TablePreflight]) -> Option<Self> {
        let mut relations = tables
            .iter()
            .filter(|table| {
                table
                    .issues
                    .iter()
                    .any(|issue| issue.starts_with("source schema fingerprint mismatch:"))
            })
            .map(trellara_pg_capture::TablePreflight::qualified_name)
            .collect::<Vec<_>>();
        relations.sort();
        relations.dedup();

        if relations.is_empty() {
            return None;
        }

        Some(Self {
            relation_count: relations.len(),
            reason:
                "configured source schema fingerprint no longer matches live pgoutput metadata"
                    .to_string(),
            recommendation:
                "pause CDC, run schema-discover and contract-test, then create a fresh snapshot-to-stream handoff before resuming"
                    .to_string(),
            relations,
        })
    }
}
