use std::collections::HashSet;

use crate::contract_recovery::contract_recovery_actions;
use crate::contract_relation_checks::{
    cdc_identity_contract_checks, pgoutput_relation_metadata_contract_checks, toast_contract_checks,
};
use crate::contract_types::*;
use crate::{
    row_filter_contract_checks, transaction_boundary_contract_checks, TrellaraConfig,
    UnknownTablePolicy,
};

impl ContractTestSummary {
    pub(crate) fn from_preflight(
        config: &TrellaraConfig,
        tables: Vec<trellara_pg_capture::TablePreflight>,
    ) -> Self {
        let mut checks = Vec::new();
        let configured_tables = config
            .dataset
            .tables
            .iter()
            .map(|table| (table.schema.as_str(), table.name.as_str()))
            .collect::<HashSet<_>>();

        for table in &tables {
            let relation = table.qualified_name();
            if !configured_tables.contains(&(table.schema.as_str(), table.name.as_str())) {
                match config.dataset.unknown_table_policy {
                    UnknownTablePolicy::Reject => checks.push(ContractCheck::failed(
                        format!("source_table_policy:{relation}"),
                        ContractSeverity::Error,
                        format!("{relation} is not listed in dataset.tables"),
                        "add the table to dataset.tables with an explicit contract or set dataset.unknown_table_policy=allow_compatible for controlled auto-adoption",
                    )),
                    UnknownTablePolicy::AllowCompatible => checks.push(
                        ContractCheck::passed_with_severity(
                            format!("source_table_policy:{relation}"),
                            ContractSeverity::Warning,
                            format!(
                                "{relation} is not listed in dataset.tables but unknown_table_policy=allow_compatible permits it"
                            ),
                        ),
                    ),
                }
            }
            if table.issues.is_empty() {
                checks.push(ContractCheck::passed(
                    format!("source_and_target_contract:{relation}"),
                    format!("{relation} satisfies source capture and target compatibility checks"),
                ));
            } else {
                for issue in &table.issues {
                    checks.push(ContractCheck::failed(
                        format!("source_and_target_contract:{relation}"),
                        ContractSeverity::Error,
                        format!("{relation}: {issue}"),
                        "repair schema, replica identity, or target compatibility before running CDC",
                    ));
                }
            }
            checks.extend(pgoutput_relation_metadata_contract_checks(config, table));
            checks.extend(cdc_identity_contract_checks(table));
            checks.extend(toast_contract_checks(table));
            for note in &table.contract_notes {
                checks.push(ContractCheck::passed_with_severity(
                    format!("schema_compatibility:{relation}"),
                    ContractSeverity::Warning,
                    format!("{relation}: {note}"),
                ));
            }
        }

        checks.extend(row_filter_contract_checks(config));
        checks.extend(transaction_boundary_contract_checks(config, &tables));

        let issue_count = checks.iter().filter(|check| !check.passed).count();
        let recovery_actions = contract_recovery_actions(config, &tables);
        Self {
            passed: issue_count == 0,
            check_count: checks.len(),
            issue_count,
            recovery_action_count: recovery_actions.len(),
            checks,
            recovery_actions,
        }
    }
}

impl ContractCheck {
    pub(crate) fn passed(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self::passed_with_severity(name, ContractSeverity::Info, message)
    }

    pub(crate) fn passed_with_severity(
        name: impl Into<String>,
        severity: ContractSeverity,
        message: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            passed: true,
            severity,
            message: message.into(),
            recommendation: String::new(),
        }
    }

    pub(crate) fn failed(
        name: impl Into<String>,
        severity: ContractSeverity,
        message: impl Into<String>,
        recommendation: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            passed: false,
            severity,
            message: message.into(),
            recommendation: recommendation.into(),
        }
    }
}
