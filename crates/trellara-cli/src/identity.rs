use std::path::Path;

use crate::{
    apply_guarantees, non_empty_string, IdentityAuditSummary, IdentityAuditTable,
    IdentityAuditTableStatus, TableConfig, TrellaraConfig,
};

impl IdentityAuditSummary {
    pub(crate) fn from_config(config: &TrellaraConfig, path: &Path) -> Self {
        let config_path = path.display().to_string();
        let tables = config
            .dataset
            .tables
            .iter()
            .map(IdentityAuditTable::from_config)
            .collect::<Vec<_>>();
        let pk_apply_ready_count = tables
            .iter()
            .filter(|table| table.status == IdentityAuditTableStatus::PkApplyReady)
            .count();
        let live_identity_review_required_count = tables.len().saturating_sub(pk_apply_ready_count);

        Self {
            source_id: config.source.id.clone(),
            dataset_id: config.dataset.id.clone(),
            config: config_path.clone(),
            table_count: tables.len(),
            pk_apply_ready_count,
            live_identity_review_required_count,
            cdc_apply_gate: identity_cdc_apply_gate(live_identity_review_required_count),
            toast_preservation_contract:
                "apply preserves absent non-key columns as unchanged so pgoutput omitted TOAST values do not overwrite target data"
                    .to_string(),
            ordinary_pk_tables_do_not_require_full: live_identity_review_required_count == 0,
            apply_guarantees: apply_guarantees(&config_path),
            tables,
            proof_commands: vec![
                format!("trellara identity-audit --config {config_path} --format text"),
                format!("trellara check --config {config_path} --format text"),
                format!("trellara contract-test --config {config_path}"),
                "cargo test -p trellara-apply-postgres plans_update_with_key_predicate"
                    .to_string(),
                "cargo test -p trellara-apply-postgres update_omits_absent_non_key_columns_for_unchanged_toast"
                    .to_string(),
                "cargo test -p trellara-apply-postgres update_omits_explicit_unchanged_toast_marker"
                    .to_string(),
                "cargo test -p trellara-apply-postgres plans_delete_with_key_predicate"
                    .to_string(),
                "cargo test -p trellara-pg-capture pgoutput_decoder_rejects_omitted_unchanged_key_column"
                    .to_string(),
            ],
            live_evidence_required: vec![
                "source-safety or contract-test shows ordinary primary-key tables on replica identity DEFAULT are warnings, not blockers".to_string(),
                "contract-test emits toast_patching warnings for TOASTable non-key columns under DEFAULT identity".to_string(),
                "apply evidence shows UPDATE and DELETE use primary-key predicates instead of full-row matching".to_string(),
                "apply evidence shows absent non-key columns are omitted from UPDATE statements so unchanged TOAST values are preserved".to_string(),
            ],
            next_commands: vec![
                format!("trellara check --config {config_path} --format text"),
                format!("trellara contract-test --config {config_path}"),
                format!("trellara pilot-package --config {config_path}"),
            ],
        }
    }
}

fn identity_cdc_apply_gate(live_identity_review_required_count: usize) -> String {
    if live_identity_review_required_count == 0 {
        "released: configured primary-key apply can run under replica identity DEFAULT after live source-safety confirms keys".to_string()
    } else {
        format!(
            "held: {live_identity_review_required_count} table(s) require live primary-key, replica identity index, or FULL-row identity review before UPDATE/DELETE CDC"
        )
    }
}

impl IdentityAuditTable {
    pub(crate) fn from_config(table: &TableConfig) -> Self {
        let verify = table.verify.clone().unwrap_or_default();
        let configured_primary_key = non_empty_string(verify.primary_key);
        let contract = table.contract.clone().unwrap_or_default();
        let relation = table.relation_id().display_name();
        let status = if configured_primary_key.is_some() {
            IdentityAuditTableStatus::PkApplyReady
        } else {
            IdentityAuditTableStatus::LiveIdentityReviewRequired
        };
        let apply_strategy = configured_primary_key
            .as_ref()
            .map(|primary_key| {
                format!("UPDATE and DELETE use key predicates on configured primary key {primary_key}")
            })
            .unwrap_or_else(|| {
                "no configured primary key; live source identity index or FULL row identity must be reviewed before UPDATE/DELETE CDC"
                    .to_string()
            });
        let replica_identity_requirement = match configured_primary_key.as_deref() {
            Some(primary_key) => format!(
                "REPLICA IDENTITY FULL is not required for ordinary rows when live source metadata confirms primary key {primary_key} under DEFAULT identity"
            ),
            None => {
                "REPLICA IDENTITY FULL or a replica identity index is required unless a primary key is configured and proven live"
                    .to_string()
            }
        };

        Self {
            relation,
            configured_primary_key,
            apply_strategy,
            replica_identity_requirement,
            toast_handling:
                "absent non-key columns are treated as unchanged; explicit nulls still apply as null"
                    .to_string(),
            target_owned_columns: contract.target_owned_columns,
            status,
        }
    }
}
