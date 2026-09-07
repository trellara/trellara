use std::collections::BTreeMap;

use crate::{
    contract_types::{ContractRecoveryAction, ContractSeverity},
    TrellaraConfig,
};

pub(crate) fn contract_recovery_actions(
    config: &TrellaraConfig,
    tables: &[trellara_pg_capture::TablePreflight],
) -> Vec<ContractRecoveryAction> {
    let mut actions = Vec::new();
    let configured_by_relation = config
        .dataset
        .tables
        .iter()
        .map(|table| (table.relation_id().display_name(), table))
        .collect::<BTreeMap<_, _>>();

    let fingerprint_drift = tables
        .iter()
        .filter(|table| {
            configured_by_relation.contains_key(&table.qualified_name())
                && table
                    .issues
                    .iter()
                    .any(|issue| issue.starts_with("source schema fingerprint mismatch:"))
        })
        .map(trellara_pg_capture::TablePreflight::qualified_name)
        .collect::<Vec<_>>();

    if !fingerprint_drift.is_empty() {
        let mut command_templates = vec![
            "trellara schema-discover --config <config>".to_string(),
            "trellara contract-test --config <config>".to_string(),
            "trellara snapshot --config <config> --force".to_string(),
            "trellara relay --config <config>".to_string(),
        ];
        if config.target.is_some() {
            command_templates.extend([
                "trellara apply --config <config>".to_string(),
                "trellara verify --config <config>".to_string(),
            ]);
        }
        actions.push(ContractRecoveryAction {
            code: "source_schema_handoff_required".to_string(),
            severity: ContractSeverity::Error,
            reason:
                "configured source schema fingerprint no longer matches live pgoutput metadata"
                    .to_string(),
            relations: fingerprint_drift,
            command_templates,
            hint: "pause CDC for the affected flow, discover the new schema fingerprint, review and update the contract, then create a fresh audited snapshot-to-stream handoff before resuming"
                .to_string(),
        });
    }

    let missing_fingerprints = configured_by_relation
        .iter()
        .filter(|(_, table)| {
            table
                .contract
                .as_ref()
                .and_then(|contract| contract.source_schema_fingerprint)
                .is_none()
        })
        .map(|(relation, _)| relation.clone())
        .collect::<Vec<_>>();

    if !missing_fingerprints.is_empty() {
        actions.push(ContractRecoveryAction {
            code: "source_schema_fingerprint_pin".to_string(),
            severity: ContractSeverity::Warning,
            relations: missing_fingerprints,
            reason: "configured source tables do not all pin pgoutput schema fingerprints"
                .to_string(),
            command_templates: vec![
                "trellara schema-discover --config <config>".to_string(),
                "trellara contract-test --config <config>".to_string(),
            ],
            hint:
                "pin dataset.tables[].contract.source_schema_fingerprint before production CDC so relation metadata drift fails closed with a scripted handoff"
                    .to_string(),
        });
    }

    actions
}
