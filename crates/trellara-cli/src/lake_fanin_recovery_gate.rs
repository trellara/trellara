pub(crate) fn non_consumable_epoch_gate(state: trellara_lake::LakeCompletenessState) -> String {
    let guidance = trellara_lake::lake_epoch_recovery_guidance(state);
    format!(
        "blocked: lake epoch state {} is not consumable; {}: {}",
        crate::lake_completeness_state_label(state),
        guidance.code,
        guidance.operator_action
    )
}
