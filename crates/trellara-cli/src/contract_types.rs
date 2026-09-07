use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct BootstrapSummary {
    pub(crate) publication: String,
    pub(crate) slot: String,
    pub(crate) consistent_lsn: Option<String>,
    pub(crate) exported_snapshot_name: Option<String>,
    pub(crate) relation_count: usize,
    pub(crate) relations: Vec<String>,
    pub(crate) preflight: PreflightSummary,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct PreflightSummary {
    pub(crate) passed: bool,
    pub(crate) issue_count: usize,
    pub(crate) tables: Vec<trellara_pg_capture::TablePreflight>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SchemaDiscoverySummary {
    pub(crate) source_id: String,
    pub(crate) dataset_id: String,
    pub(crate) table_count: usize,
    pub(crate) tables: Vec<SchemaDiscoveryTable>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct SchemaDiscoveryTable {
    pub(crate) relation: String,
    pub(crate) configured: bool,
    pub(crate) exists: bool,
    pub(crate) replica_identity: Option<String>,
    pub(crate) update_delete_safe: bool,
    pub(crate) primary_key_columns: Vec<String>,
    pub(crate) column_count: usize,
    pub(crate) schema_fingerprint: Option<u64>,
    pub(crate) issues: Vec<String>,
    pub(crate) contract_notes: Vec<String>,
    pub(crate) suggested_primary_key: Option<String>,
    pub(crate) config_snippet: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ContractTestSummary {
    pub(crate) passed: bool,
    pub(crate) check_count: usize,
    pub(crate) issue_count: usize,
    pub(crate) recovery_action_count: usize,
    pub(crate) checks: Vec<ContractCheck>,
    pub(crate) recovery_actions: Vec<ContractRecoveryAction>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ContractCheck {
    pub(crate) name: String,
    pub(crate) passed: bool,
    pub(crate) severity: ContractSeverity,
    pub(crate) message: String,
    pub(crate) recommendation: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ContractRecoveryAction {
    pub(crate) code: String,
    pub(crate) severity: ContractSeverity,
    pub(crate) relations: Vec<String>,
    pub(crate) reason: String,
    pub(crate) command_templates: Vec<String>,
    pub(crate) hint: String,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ContractSeverity {
    Info,
    Warning,
    Error,
}
