use super::*;

pub(super) struct LocalReplayFixture {
    pub(super) config: TrellaraConfig,
    root: PathBuf,
}

impl Drop for LocalReplayFixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(super) fn strict_fixture(prefix: &str) -> LocalReplayFixture {
    let root = temp_root(prefix);
    let yaml = local_stream_yaml().replace(
        "  path: /tmp/trellara-local-stream",
        &format!("  path: {}", root.display()),
    );
    let config = TrellaraConfig::from_yaml(&yaml, "test").expect("parse");

    LocalReplayFixture { config, root }
}

pub(super) fn partitioned_fixture(prefix: &str) -> LocalReplayFixture {
    let root = temp_root(prefix);
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

    LocalReplayFixture { config, root }
}

pub(super) fn strict_topic() -> String {
    "trellara.local-source.retail-sales.strict".to_string()
}

pub(super) fn seek_args(topic: String, next_offset: i64) -> LocalStreamSeekArgs {
    LocalStreamSeekArgs {
        config: PathBuf::from("local.yml"),
        topic,
        next_offset,
        transaction_id: None,
        commit_lsn: None,
        consumer_group: None,
        allow_ahead: false,
    }
}

pub(super) fn exact_seek_args(
    topic: String,
    next_offset: i64,
    transaction_id: &str,
    commit_lsn: &str,
) -> LocalStreamSeekArgs {
    LocalStreamSeekArgs {
        transaction_id: Some(transaction_id.to_string()),
        commit_lsn: Some(commit_lsn.to_string()),
        ..seek_args(topic, next_offset)
    }
}

fn temp_root(prefix: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("{prefix}-{}", unique_test_suffix()));
    let _ = std::fs::remove_dir_all(&root);
    root
}
