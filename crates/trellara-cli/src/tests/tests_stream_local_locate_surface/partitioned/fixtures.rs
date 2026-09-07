use super::*;

pub(super) struct PartitionedLocateFixture {
    pub(super) config: TrellaraConfig,
    root: PathBuf,
}

impl Drop for PartitionedLocateFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(super) fn partitioned_fixture(prefix: &str) -> PartitionedLocateFixture {
    let root = std::env::temp_dir().join(format!("{prefix}-{}", unique_test_suffix()));
    let _ = std::fs::remove_dir_all(&root);
    let yaml = local_stream_yaml()
        .replace("strict_transaction_order", "partitioned_scale_mode")
        .replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n  partition:\n    partition_count: 2\n    key_column: store_id",
        )
        .replace(
            "  path: /tmp/trellara-local-stream",
            &format!("  path: {}", root.display()),
        );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    PartitionedLocateFixture { config, root }
}

pub(super) fn partitioned_locate_args(transaction_id: &str) -> LocalStreamLocateArgs {
    LocalStreamLocateArgs {
        config: PathBuf::from("local.yml"),
        transaction_id: transaction_id.to_string(),
        commit_lsn: Some("0/16B6C90".to_string()),
        topic: None,
    }
}
