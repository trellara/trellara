use super::*;

pub(in crate::tests) const STRICT_YAML: &str = r#"
source:
  id: local-source
  database_url: postgresql://trellara:trellara@localhost:55432/trellara_source
  publication: trellara_retail
  slot: trellara_retail_slot
dataset:
  id: retail-sales
  mode: strict_transaction_order
  tables:
    - schema: public
      name: sales
stream:
  kind: kafka
  bootstrap_servers: localhost:9092
  topic: trellara.local-source.retail-sales.strict
target:
  database_url: postgresql://trellara:trellara@localhost:55433/trellara_target
"#;

pub(in crate::tests) fn local_stream_yaml() -> String {
    STRICT_YAML.replace(
            "stream:\n  kind: kafka\n  bootstrap_servers: localhost:9092\n  topic: trellara.local-source.retail-sales.strict",
            "stream:\n  kind: local\n  path: /tmp/trellara-local-stream",
        )
}

pub(in crate::tests) fn local_strict_chunking_yaml() -> String {
    local_stream_yaml().replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  strict_chunking:\n    max_changes_per_chunk: 1000\n  tables:\n    - schema: public\n      name: sales",
        )
}

pub(in crate::tests) fn multi_table_strict_yaml() -> String {
    STRICT_YAML.replace(
            "  tables:\n    - schema: public\n      name: sales",
            "  tables:\n    - schema: public\n      name: sales\n    - schema: public\n      name: payments",
        )
}

pub(in crate::tests) fn partitioned_yaml() -> String {
    STRICT_YAML
            .replace(
                "mode: strict_transaction_order",
                "mode: partitioned_scale_mode\n  partition:\n    partition_count: 16\n    key_column: store_id\n    null_key_policy: quarantine\n    key_change_policy: quarantine",
            )
            .replace(
                "topic: trellara.local-source.retail-sales.strict",
                "topic: trellara.local-source.retail-sales.partitioned",
            )
}

pub(in crate::tests) fn local_partitioned_yaml() -> String {
    local_stream_yaml().replace(
            "mode: strict_transaction_order",
            "mode: partitioned_scale_mode\n  partition:\n    partition_count: 16\n    key_column: store_id\n    null_key_policy: quarantine\n    key_change_policy: quarantine",
        )
}

pub(in crate::tests) fn workspace_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}
