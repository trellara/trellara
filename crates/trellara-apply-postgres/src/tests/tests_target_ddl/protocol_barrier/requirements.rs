use super::*;

#[test]
fn target_ddl_barrier_from_envelope_with_partitioned_requirements_pauses_visibility() {
    let envelope = ddl_envelope();
    let barrier = target_ddl_barrier_from_envelope_with_requirements(
        &envelope,
        TargetDdlBarrierRequirements::partitioned_scale(vec![
            "target_postgres".to_string(),
            "raw_cdc_lake".to_string(),
        ]),
    )
    .expect("DDL barrier")
    .expect("DDL events");

    assert_eq!(
        barrier.required_sinks,
        vec!["target_postgres", "raw_cdc_lake", "partition_visibility"]
    );
    assert!(barrier.requires_global_partition_pause);
}

#[test]
fn target_ddl_barrier_requirements_normalize_plan_sinks() {
    let requirements = TargetDdlBarrierRequirements::from_required_sinks(
        [
            "raw_cdc_lake",
            "",
            " raw_cdc_lake ",
            "target_postgres",
            "raw_cdc_lake",
            " spark_derived_views ",
            "spark_derived_views",
        ],
        false,
    );

    assert_eq!(
        requirements.required_sinks,
        vec!["raw_cdc_lake", "target_postgres", "spark_derived_views"]
    );
    assert!(!requirements.requires_global_partition_pause);
}

#[test]
fn target_ddl_barrier_requirements_always_include_target_postgres() {
    let requirements = TargetDdlBarrierRequirements::from_required_sinks(["raw_cdc_lake"], false);

    assert_eq!(
        requirements.required_sinks,
        vec!["raw_cdc_lake", "target_postgres"]
    );
}

#[test]
fn partitioned_requirements_dedupe_existing_partition_visibility_sink() {
    let requirements = TargetDdlBarrierRequirements::partitioned_scale(vec![
        "target_postgres".to_string(),
        " partition_visibility ".to_string(),
    ]);

    assert_eq!(
        requirements.required_sinks,
        vec!["target_postgres", "partition_visibility"]
    );
    assert!(requirements.requires_global_partition_pause);
}
