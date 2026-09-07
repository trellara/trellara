use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct IdentityApplyGuarantee {
    pub(crate) code: String,
    pub(crate) guarantee: String,
    pub(crate) transaction_boundary: String,
    pub(crate) proof_command: String,
}

pub(crate) fn apply_guarantees(config_path: &str) -> Vec<IdentityApplyGuarantee> {
    vec![
        IdentityApplyGuarantee {
            code: "default_identity_pk_apply".to_string(),
            guarantee:
                "ordinary primary-key tables on REPLICA IDENTITY DEFAULT are advisory, not blockers; FULL is not required when live metadata proves key-addressable rows"
                    .to_string(),
            transaction_boundary:
                "accepted only when source-safety or contract-test proves the key before CDC apply begins"
                    .to_string(),
            proof_command: format!("trellara check --config {config_path} --format text"),
        },
        IdentityApplyGuarantee {
            code: "key_predicate_update_delete".to_string(),
            guarantee:
                "UPDATE and DELETE apply with primary-key predicates instead of full-row matching"
                    .to_string(),
            transaction_boundary:
                "old key image identifies the target row for key-changing updates; new key values are applied only with the committed change"
                    .to_string(),
            proof_command:
                "cargo test -p trellara-apply-postgres plans_update_with_key_predicate && cargo test -p trellara-apply-postgres key_changing_update_sets_new_key_and_matches_old_key && cargo test -p trellara-apply-postgres plans_delete_with_key_predicate"
                    .to_string(),
        },
        IdentityApplyGuarantee {
            code: "unchanged_toast_preservation".to_string(),
            guarantee:
                "omitted pgoutput TOAST values are treated as unchanged for non-key columns and are never written as null or placeholder values"
                    .to_string(),
            transaction_boundary:
                "row images that omit key columns fail closed before apply, while absent non-key columns remain omitted inside the committed transaction"
                    .to_string(),
            proof_command:
                "cargo test -p trellara-apply-postgres update_omits_absent_non_key_columns_for_unchanged_toast && cargo test -p trellara-apply-postgres update_omits_explicit_unchanged_toast_marker && cargo test -p trellara-pg-capture pgoutput_decoder_rejects_omitted_unchanged_key_column"
                    .to_string(),
        },
    ]
}
