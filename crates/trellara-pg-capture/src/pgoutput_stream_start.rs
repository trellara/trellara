use crate::replication_protocol::replication_start_lsn;
use crate::{LogicalSlotBootstrap, PgCaptureConfig, Result};

pub(crate) struct PgOutputReplicationStartPlan {
    pub(crate) start_lsn: String,
    protocol_version: String,
    streaming: String,
}

impl PgOutputReplicationStartPlan {
    pub(crate) fn options<'a>(
        &'a self,
        config: &'a PgCaptureConfig,
    ) -> Vec<(&'static str, &'a str)> {
        config.pgoutput.start_options(
            &config.publication_name,
            &self.protocol_version,
            &self.streaming,
        )
    }
}

pub(crate) fn pgoutput_replication_start_plan(
    slot: &LogicalSlotBootstrap,
    config: &PgCaptureConfig,
) -> Result<PgOutputReplicationStartPlan> {
    Ok(PgOutputReplicationStartPlan {
        start_lsn: replication_start_lsn(slot)?,
        protocol_version: config.pgoutput.protocol_version.to_string(),
        streaming: config.pgoutput.streaming.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PgOutputProtocolConfig, TableSelector};

    #[test]
    fn start_plan_binds_slot_lsn_and_pgoutput_options() {
        let config = config(PgOutputProtocolConfig {
            protocol_version: 2,
            streaming: true,
        });
        let slot = LogicalSlotBootstrap {
            created: false,
            consistent_lsn: Some("0/16B6C50".to_string()),
        };

        let plan = pgoutput_replication_start_plan(&slot, &config).expect("start plan");

        assert_eq!(plan.start_lsn, "0/16B6C50");
        assert_eq!(
            plan.options(&config),
            vec![
                ("proto_version", "2"),
                ("publication_names", "trellara_pub"),
                ("streaming", "true")
            ]
        );
    }

    #[test]
    fn start_plan_omits_streaming_option_for_protocol_v1() {
        let config = config(PgOutputProtocolConfig {
            protocol_version: 1,
            streaming: false,
        });
        let slot = LogicalSlotBootstrap {
            created: true,
            consistent_lsn: Some("0/16B6C50".to_string()),
        };

        let plan = pgoutput_replication_start_plan(&slot, &config).expect("start plan");

        assert_eq!(
            plan.options(&config),
            vec![
                ("proto_version", "1"),
                ("publication_names", "trellara_pub")
            ]
        );
    }

    #[test]
    fn start_plan_requires_slot_lsn() {
        let config = config(PgOutputProtocolConfig::default());
        let slot = LogicalSlotBootstrap {
            created: false,
            consistent_lsn: None,
        };

        assert!(pgoutput_replication_start_plan(&slot, &config).is_err());
    }

    fn config(pgoutput: PgOutputProtocolConfig) -> PgCaptureConfig {
        PgCaptureConfig {
            connection_uri: "postgres://user@localhost/db".to_string(),
            source_id: "source-a".to_string(),
            database_id: "database-a".to_string(),
            dataset_id: "dataset-a".to_string(),
            publication_name: "trellara_pub".to_string(),
            slot_name: "trellara_slot".to_string(),
            tables: vec![TableSelector::new("public", "sales")],
            create_if_missing: true,
            stream_spill_threshold_changes: 1_024,
            stream_spill_dir: None,
            pgoutput,
        }
    }
}
