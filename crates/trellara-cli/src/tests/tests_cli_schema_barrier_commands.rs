use super::*;

#[test]
fn cli_parses_schema_ddl_barrier_record_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "ddl-barrier",
        "record",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--change",
        "add_nullable_column:public.sales.discount_code:text",
        "--barrier-lsn",
        "0/16B8000",
        "--schema-version",
        "schema-v2",
        "--format",
        "text",
    ])
    .expect("cli parse");

    let Command::Schema {
        command: SchemaCommand::DdlBarrier { command },
    } = cli.command
    else {
        panic!("schema ddl-barrier command");
    };
    let DdlBarrierCommand::Record(args) = *command else {
        panic!("record command");
    };
    assert_eq!(args.barrier_lsn, "0/16B8000");
    assert_eq!(args.schema_version, "schema-v2");
    assert_eq!(args.format, QuickstartOutputFormat::Text);
}

#[test]
fn cli_parses_schema_ddl_barrier_ack_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "ddl-barrier",
        "ack",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--barrier-id",
        "ddl-barrier-abc",
        "--sink",
        "target_postgres",
        "--ack-lsn",
        "0/16B9000",
        "--schema-version",
        "schema-v2",
        "--accepted",
        "false",
        "--detail",
        "target rejected schema",
        "--plan-sha256",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "--statement-sha256",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "--barrier-lsn",
        "0/16B8000",
        "--manifest-digest",
        "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        "--template-digest",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        "--accepted-by",
        "platform-review",
        "--view-count",
        "2",
    ])
    .expect("cli parse");

    let Command::Schema {
        command: SchemaCommand::DdlBarrier { command },
    } = cli.command
    else {
        panic!("schema ddl-barrier command");
    };
    let DdlBarrierCommand::Ack(args) = *command else {
        panic!("ack command");
    };
    assert_eq!(args.barrier_id, "ddl-barrier-abc");
    assert_eq!(args.sink, "target_postgres");
    assert!(!args.accepted);
    assert_eq!(
        args.plan_sha256.as_deref(),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert_eq!(args.statement_sha256s.len(), 1);
    assert_eq!(args.barrier_lsn.as_deref(), Some("0/16B8000"));
    assert_eq!(
        args.manifest_digest.as_deref(),
        Some("abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789")
    );
    assert_eq!(
        args.template_digest.as_deref(),
        Some("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
    );
}

#[test]
fn cli_parses_schema_ddl_barrier_status_command() {
    let cli = Cli::try_parse_from([
        "trellara",
        "schema",
        "ddl-barrier",
        "status",
        "--config",
        "examples/retail-fleet/strict.yml",
        "--barrier-id",
        "ddl-barrier-abc",
    ])
    .expect("cli parse");

    let Command::Schema {
        command: SchemaCommand::DdlBarrier { command },
    } = cli.command
    else {
        panic!("schema ddl-barrier command");
    };
    let DdlBarrierCommand::Status(args) = *command else {
        panic!("status command");
    };
    assert_eq!(args.barrier_id, "ddl-barrier-abc");
}
