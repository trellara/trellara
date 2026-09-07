use super::*;

#[test]
fn fleet_init_writes_valid_local_flow_package() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-init-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let args = fleet_init_args(root.clone());

    let summary = init_fleet_configs(&args).expect("init fleet");

    assert_eq!(summary.fleet_id, "design-partner-fleet");
    assert_eq!(summary.flow_count, 2);
    assert_eq!(summary.config_count, 2);
    assert!(Path::new(&summary.manifest).exists());
    assert!(Path::new(&summary.readme).exists());
    assert_eq!(summary.configs.len(), 2);
    assert_eq!(summary.configs[0].dataset_id, "customer-east");
    assert!(summary.configs[0]
        .config
        .ends_with("configs/customer-east.yml"));
    assert_eq!(
        summary.configs[0].source_slot,
        "trellara_customer_east_slot"
    );
    assert_eq!(summary.configs[0].table_count, 2);
    assert!(summary.next_commands[0].contains("trellara fleet report --config"));
    assert!(summary.next_commands[0].contains("configs/customer-east.yml"));
    assert!(summary.next_commands[0].contains("configs/customer-west.yml"));
    assert!(summary.next_commands.contains(&format!(
        "trellara evaluate --config {} --format text",
        summary.configs[0].config
    )));
    assert!(summary.next_commands.contains(&format!(
        "trellara pilot-package --config {}",
        summary.configs[0].config
    )));

    let first_config =
        TrellaraConfig::from_path(Path::new(&summary.configs[0].config)).expect("first config");
    first_config.validate().expect("valid first config");
    assert_eq!(first_config.source.id, "store-fleet");
    assert_eq!(first_config.source.database_id.as_deref(), Some("retail"));
    assert_eq!(first_config.source.publication, "trellara_customer_east");
    assert_eq!(first_config.source.slot, "trellara_customer_east_slot");
    assert_eq!(first_config.dataset.id, "customer-east");
    assert_eq!(first_config.dataset.tables.len(), 2);
    assert!(matches!(first_config.stream, StreamConfig::Local { .. }));
    assert_eq!(
        first_config.target.as_ref().expect("target").database_url,
        "postgresql://target/app"
    );

    let readme = fs::read_to_string(&summary.readme).expect("read fleet readme");
    assert!(readme.contains("Trellara Fleet Init"));
    assert!(readme.contains("customer-east"));
    assert!(readme.contains("trellara fleet report"));
    let manifest = fs::read_to_string(&summary.manifest).expect("read fleet manifest");
    assert!(manifest.contains("\"flow_count\": 2"));
    assert!(manifest.contains("customer-west.yml"));

    assert!(matches!(
        init_fleet_configs(&args),
        Err(CliError::InvalidConfig(message)) if message.contains("already exists")
    ));
    let mut forced = args;
    forced.force = true;
    init_fleet_configs(&forced).expect("forced fleet overwrite");

    fs::remove_dir_all(root).expect("remove fleet init temp dir");
}

#[test]
fn fleet_init_rejects_duplicate_dataset_slugs() {
    let root = std::env::temp_dir().join(format!(
        "trellara-fleet-init-duplicate-{}-{}",
        std::process::id(),
        unique_test_suffix()
    ));
    let mut args = fleet_init_args(root.clone());
    args.dataset = vec!["customer.east".to_string(), "customer-east".to_string()];

    assert!(matches!(
        init_fleet_configs(&args),
        Err(CliError::InvalidConfig(message)) if message.contains("duplicate config slug")
    ));

    let _ = fs::remove_dir_all(root);
}
