use serde::Serialize;
use trellara_protocol::{DdlEvent, RelationSchemaVersion, TransactionBoundaryKind};

use crate::{protocol_ddl_operation_label, protocol_transaction_boundary_kind_label};

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlEnvelopeSchemaVersionSummary {
    pub(crate) relation: String,
    pub(crate) version: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct DdlEnvelopeEventSummary {
    pub(crate) total_order: u32,
    pub(crate) operation: String,
    pub(crate) relation: String,
    pub(crate) target_auto_apply: bool,
    pub(crate) release_gate: String,
}

pub(crate) fn transaction_boundary_kind_label(kind: TransactionBoundaryKind) -> &'static str {
    protocol_transaction_boundary_kind_label(kind)
}

pub(crate) fn schema_version_summary(
    schema_version: &RelationSchemaVersion,
) -> DdlEnvelopeSchemaVersionSummary {
    DdlEnvelopeSchemaVersionSummary {
        relation: schema_version
            .relation
            .as_ref()
            .map(|relation| relation.display_name())
            .unwrap_or_else(|| "<unknown>".to_string()),
        version: schema_version.version,
    }
}

pub(crate) fn ddl_event_summary(event: &DdlEvent) -> DdlEnvelopeEventSummary {
    DdlEnvelopeEventSummary {
        total_order: event.total_order,
        operation: protocol_ddl_operation_label(event).to_string(),
        relation: event
            .relation
            .as_ref()
            .map(|relation| relation.display_name())
            .unwrap_or_else(|| "<unknown>".to_string()),
        target_auto_apply: event.target_auto_apply,
        release_gate: event.release_gate.clone(),
    }
}
