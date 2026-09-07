use super::*;

#[test]
fn hook_registration_plan_requires_all_native_data_plane_hooks() {
    let plan = native_hook_registration_plan(true, true, true);

    assert_eq!(plan.contract, HOOK_REGISTRATION_CONTRACT);
    assert!(plan.data_plane_hooks_ready);
    assert_eq!(plan.steps.len(), 3);
    assert_eq!(plan.steps[0].name, "request_addin_shmem_space");
    assert_eq!(plan.steps[1].name, "register_background_worker");
    assert_eq!(plan.steps[2].name, "logical_decoding_callbacks");
}

#[test]
fn hook_registration_plan_fails_closed_until_every_hook_is_wired() {
    for plan in [
        native_hook_registration_plan(false, true, true),
        native_hook_registration_plan(true, false, true),
        native_hook_registration_plan(true, true, false),
    ] {
        assert!(!plan.data_plane_hooks_ready);
    }
}
