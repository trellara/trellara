use super::*;

#[test]
fn source_capture_kind_parser_rejects_unknown_capture_plugins() {
    assert_eq!(
        parse_source_capture_kind("pgoutput").expect("pgoutput"),
        SourceCaptureKind::PgOutput
    );
    assert_eq!(
        parse_source_capture_kind("test_decoding").expect("test_decoding"),
        SourceCaptureKind::TestDecoding
    );
    assert!(matches!(
        parse_source_capture_kind("wal2json"),
        Err(CliError::InvalidConfig(message)) if message.contains("pgoutput or test_decoding")
    ));
}

#[test]
fn direct_source_safety_capture_config_accepts_managed_tls_urls() {
    for database_url in [
        "postgresql://source/app?sslmode=require",
        "postgresql://source/app?connect_timeout=3&sslmode=require",
        "host=db.example.com user=trellara dbname=retail sslmode=require",
    ] {
        let mut args = direct_source_safety_args();
        args.database_url = Some(database_url.to_string());

        let config = direct_source_safety_capture_config(&args).expect("managed TLS config");

        assert_eq!(config.capture_config.connection_uri, database_url);
    }
}

#[test]
fn direct_source_safety_capture_config_is_read_only() {
    let args = direct_source_safety_args();

    let config = direct_source_safety_capture_config(&args).expect("direct source config");

    assert_eq!(config.capture_kind, SourceCaptureKind::PgOutput);
    assert!(!config.capture_config.create_if_missing);
    assert_eq!(config.capture_config.publication_name, "trellara_pub");
    assert_eq!(config.capture_config.slot_name, "trellara_slot");
    assert_eq!(config.capture_config.tables.len(), 1);
}

#[test]
fn direct_source_safety_capture_config_rejects_padded_identity_fields() {
    for field in [
        "database_url",
        "source_id",
        "database_id",
        "dataset_id",
        "publication",
        "slot",
        "capture",
    ] {
        let mut args = direct_source_safety_args();
        match field {
            "database_url" => {
                args.database_url = Some(" postgresql://source/app?sslmode=prefer".to_string());
            }
            "source_id" => args.source_id = " source-a".to_string(),
            "database_id" => args.database_id = "postgres ".to_string(),
            "dataset_id" => args.dataset_id = "\tretail".to_string(),
            "publication" => args.publication = " trellara_pub".to_string(),
            "slot" => args.slot = "trellara_slot ".to_string(),
            "capture" => args.capture = " pgoutput".to_string(),
            _ => unreachable!("test field is exhaustive"),
        }

        assert!(matches!(
            direct_source_safety_capture_config(&args),
            Err(CliError::InvalidConfig(message))
                if message == format!("{field} must not contain surrounding whitespace")
        ));
    }
}

#[test]
fn direct_source_safety_capture_config_rejects_padded_table_identity() {
    for table in [
        " public.sales",
        "public.sales ",
        "public. sales",
        "public .sales",
    ] {
        let mut args = direct_source_safety_args();
        args.table = vec![table.to_string()];

        assert!(matches!(
            direct_source_safety_capture_config(&args),
            Err(CliError::InvalidConfig(message))
                if message.contains("must not contain surrounding whitespace")
        ));
    }
}

#[test]
fn direct_source_safety_capture_config_rejects_missing_write_init_target_before_connecting() {
    let mut args = direct_source_safety_args();
    args.write_init = Some(PathBuf::from("trellara.yml"));

    assert!(matches!(
        direct_source_safety_capture_config(&args),
        Err(CliError::InvalidConfig(message))
            if message == "check --write-init requires --target-database-url"
    ));
}

#[test]
fn direct_source_safety_capture_config_rejects_padded_write_init_target_before_connecting() {
    let mut args = direct_source_safety_args();
    args.target_database_url = Some(" postgresql://target/app".to_string());
    args.write_init = Some(PathBuf::from("trellara.yml"));

    assert!(matches!(
        direct_source_safety_capture_config(&args),
        Err(CliError::InvalidConfig(message))
            if message == "target_database_url must not contain surrounding whitespace"
    ));
}

fn direct_source_safety_args() -> SourceSafetyArgs {
    SourceSafetyArgs {
        config: None,
        format: SourceSafetyOutputFormat::Json,
        output: None,
        database_url: Some("postgresql://source/app?sslmode=prefer".to_string()),
        target_database_url: None,
        source_id: "source-a".to_string(),
        database_id: "postgres".to_string(),
        dataset_id: "retail".to_string(),
        publication: "trellara_pub".to_string(),
        slot: "trellara_slot".to_string(),
        capture: "pgoutput".to_string(),
        wal_retention_warn_bytes: Some(1024),
        table: vec!["public.sales".to_string()],
        write_init: None,
        force: false,
    }
}
