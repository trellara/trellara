use super::fixtures::{ddl_envelope, manual_review_ddl_envelope, mixed_ddl_dml_envelope};
use super::*;

#[tokio::test]
async fn schema_ddl_envelope_plan_command_renders_runtime_sequence() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-envelope-plan-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    let envelope_path = root.join("ddl-envelope.pb");
    fs::create_dir_all(&root).expect("create ddl envelope plan temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    fs::write(
        &envelope_path,
        ddl_envelope().encode_checked().expect("encode envelope"),
    )
    .expect("write envelope");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlEnvelopePlan(DdlEnvelopePlanArgs {
                config: config_path,
                file: envelope_path,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("ddl envelope plan output");

    assert!(output.contains("Trellara DDL envelope runtime plan"));
    assert!(
        output.contains("source_boundary_kind=ddl_only ddl_events=1 dml_changes=0 executable=true")
    );
    assert!(output.contains("barrier_schema_version=schema-fingerprint:67890"));
    assert!(output.contains("propagation_boundary=source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks"));
    assert!(output.contains("propagation_policy_sha256="));
    assert!(output.contains("propagation_decisions:"));
    assert!(output.contains("auto_apply:1"));
    assert!(output.contains("target_ack_required:1"));
    assert!(output.contains("schema_version_evidence:"));
    assert!(output.contains("relation=public.sales version=67890"));
    assert!(output.contains("required_sinks:"));
    assert!(output.contains("target_postgres"));
    assert!(output.contains("raw_cdc_lake"));
    assert!(output.contains(
        "order=1 operation=add_column relation=public.sales auto_apply=true release_gate=post_ddl_dml_release"
    ));
    assert!(output.contains(
        "ALTER TABLE \"public\".\"sales\" ADD COLUMN IF NOT EXISTS \"discount_code\" text;"
    ));
    assert!(output.contains("record envelope-derived DDL barrier"));
    assert!(output.contains("apply 0 DML changes from a replay envelope with ddl_events stripped"));

    fs::remove_dir_all(root).expect("remove ddl envelope plan temp dir");
}

#[tokio::test]
async fn schema_ddl_envelope_plan_command_exposes_dml_replay_projection() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-envelope-plan-replay-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    let envelope_path = root.join("ddl-envelope.pb");
    fs::create_dir_all(&root).expect("create replay ddl envelope plan temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    fs::write(
        &envelope_path,
        mixed_ddl_dml_envelope()
            .encode_checked()
            .expect("encode envelope"),
    )
    .expect("write envelope");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlEnvelopePlan(DdlEnvelopePlanArgs {
                config: config_path,
                file: envelope_path,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("ddl envelope replay plan output");

    assert!(output.contains("source_boundary_kind=mixed_ddl_and_dml"));
    assert!(output.contains("dml_replay_after_barrier:"));
    assert!(output.contains(
        "barrier_id=source-a:retail:sales:tx-mixed-ddl:0/16B6C50:ddl release_gate=post_ddl_dml_release dml_changes=1 ddl_events=0"
    ));
    assert!(output.contains("ddl_events_stripped=true replay_allowed=false"));
    assert!(output.contains(
        "blocked_until=ddl-barrier status for source-a:retail:sales:tx-mixed-ddl:0/16B6C50:ddl reports post_ddl_dml_release satisfied"
    ));
    assert!(output.contains("apply 1 DML changes from a replay envelope with ddl_events stripped"));

    fs::remove_dir_all(root).expect("remove replay ddl envelope plan temp dir");
}

#[tokio::test]
async fn schema_ddl_envelope_plan_json_exposes_replay_barrier_id() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-envelope-plan-replay-json-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    let envelope_path = root.join("ddl-envelope.pb");
    fs::create_dir_all(&root).expect("create replay ddl envelope plan json temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    fs::write(
        &envelope_path,
        mixed_ddl_dml_envelope()
            .encode_checked()
            .expect("encode envelope"),
    )
    .expect("write envelope");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlEnvelopePlan(DdlEnvelopePlanArgs {
                config: config_path,
                file: envelope_path,
                format: QuickstartOutputFormat::Json,
            }),
        },
    })
    .await
    .expect("ddl envelope replay plan json output");
    let json: serde_json::Value = serde_json::from_str(&output).expect("json output");

    assert_eq!(
        json["dml_replay_after_barrier"]["barrier_id"],
        "source-a:retail:sales:tx-mixed-ddl:0/16B6C50:ddl"
    );
    assert_eq!(
        json["propagation_boundary"],
        "source_commit_lsn_barrier_holds_post_ddl_dml_until_required_sink_acks"
    );
    assert_eq!(json["propagation_decisions"][0], "auto_apply:1");
    assert_eq!(json["propagation_decisions"][3], "target_ack_required:1");
    assert_eq!(
        json["propagation_policy_sha256"]
            .as_str()
            .expect("policy digest")
            .len(),
        64
    );
    assert_eq!(
        json["dml_replay_after_barrier"]["blocked_until"],
        "ddl-barrier status for source-a:retail:sales:tx-mixed-ddl:0/16B6C50:ddl reports post_ddl_dml_release satisfied"
    );

    fs::remove_dir_all(root).expect("remove replay ddl envelope plan json temp dir");
}

#[tokio::test]
async fn schema_ddl_envelope_plan_command_renders_manual_review_blockers() {
    let root = std::env::temp_dir().join(format!(
        "trellara-schema-ddl-envelope-plan-blocked-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let config_path = root.join("strict.yml");
    let envelope_path = root.join("ddl-envelope.pb");
    fs::create_dir_all(&root).expect("create blocked ddl envelope plan temp dir");
    fs::write(&config_path, STRICT_YAML).expect("write config");
    fs::write(
        &envelope_path,
        manual_review_ddl_envelope()
            .encode_checked()
            .expect("encode envelope"),
    )
    .expect("write envelope");

    let output = execute(Cli {
        command: Command::Schema {
            command: SchemaCommand::DdlEnvelopePlan(DdlEnvelopePlanArgs {
                config: config_path,
                file: envelope_path,
                format: QuickstartOutputFormat::Text,
            }),
        },
    })
    .await
    .expect("ddl envelope blocked plan output");

    assert!(output
        .contains("source_boundary_kind=ddl_only ddl_events=1 dml_changes=0 executable=false"));
    assert!(output.contains(
        "order=1 operation=change_partition_key relation=public.sales auto_apply=false release_gate=post_ddl_dml_release"
    ));
    assert!(output.contains("requires manual review"));
    assert!(output.contains("target_sql:\n- none"));
    assert!(output.contains("barrier_id=source-a:retail:sales:tx-ddl:0/16B6C50:ddl"));

    fs::remove_dir_all(root).expect("remove blocked ddl envelope plan temp dir");
}
