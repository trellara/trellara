use super::*;

pub(crate) fn config() -> TrellaraConfig {
    TrellaraConfig::from_yaml(STRICT_YAML, "test").expect("parse config")
}

pub(crate) fn base_args(sink: &str) -> DdlBarrierAckArgs {
    DdlBarrierAckArgs {
        config: PathBuf::from("strict.yml"),
        barrier_id: "ddl-barrier-123".to_string(),
        sink: sink.to_string(),
        ack_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        accepted: true,
        detail: "accepted".to_string(),
        plan_sha256: None,
        statement_sha256s: Vec::new(),
        epoch_id: None,
        metadata_table: None,
        partition_metadata_table: None,
        manifest_digest: None,
        template_digest: None,
        accepted_by: None,
        view_count: None,
        barrier_lsn: None,
        expected_partition_count: None,
        partition_durable_lsns: Vec::new(),
        partition_applied_lsns: Vec::new(),
        format: QuickstartOutputFormat::Json,
    }
}

pub(crate) fn shared_barrier() -> DdlBarrier {
    DdlBarrier {
        source_id: "local-source".to_string(),
        database_id: "retail-sales".to_string(),
        dataset_id: "retail-sales".to_string(),
        barrier_id: "ddl-barrier-123".to_string(),
        barrier_lsn: "0/16B9000".to_string(),
        schema_version: "schema-v2".to_string(),
        cdc_transaction_boundary: "source commit LSN is the DDL barrier; post-DDL DML stays invisible until required acknowledgements reach barrier_lsn".to_string(),
        required_sinks: vec!["raw_cdc_lake".to_string()],
        requires_global_partition_pause: false,
    }
}
