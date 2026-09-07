use super::*;

pub(in crate::tests) fn init_args(output: PathBuf) -> InitArgs {
    InitArgs {
        source_database_url: "postgresql://source/app".to_string(),
        target_database_url: Some("postgresql://target/app".to_string()),
        source_id: "store-fleet".to_string(),
        database_id: "retail".to_string(),
        dataset_id: "sales".to_string(),
        publication: "trellara_sales".to_string(),
        slot: "trellara_sales_slot".to_string(),
        stream_path: PathBuf::from("./target/trellara-local-stream"),
        output,
        primary_key: "id".to_string(),
        table: vec!["public.sales".to_string(), "public.payments".to_string()],
        evaluate: false,
        force: false,
    }
}

pub(in crate::tests) fn fleet_init_args(output_dir: PathBuf) -> FleetInitArgs {
    FleetInitArgs {
        source_database_url: "postgresql://source/app".to_string(),
        target_database_url: Some("postgresql://target/app".to_string()),
        fleet_id: "design-partner-fleet".to_string(),
        source_id: "store-fleet".to_string(),
        database_id: "retail".to_string(),
        dataset: vec!["customer-east".to_string(), "customer-west".to_string()],
        table: vec!["public.sales".to_string(), "public.payments".to_string()],
        primary_key: "id".to_string(),
        output_dir,
        force: false,
    }
}
