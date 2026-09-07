use tracing::debug;

use crate::replication::ReplicationBootstrapConnection;
use crate::{
    CaptureError, ExportedLogicalSlot, LogicalSlotBootstrap, PgCapture, PgCaptureConfig, Result,
};

impl PgCapture {
    pub async fn ensure_logical_slot(&self) -> Result<LogicalSlotBootstrap> {
        let existing_lsn = self
            .client
            .query_opt(
                r#"
                select coalesce(confirmed_flush_lsn, restart_lsn)::text
                  from pg_replication_slots
                 where slot_name = $1
                "#,
                &[&self.config.slot_name],
            )
            .await?
            .map(|row| row.get::<_, Option<String>>(0));

        if let Some(consistent_lsn) = existing_lsn {
            debug!(slot = %self.config.slot_name, "logical replication slot already exists");
            return Ok(LogicalSlotBootstrap {
                created: false,
                consistent_lsn,
            });
        }

        if !self.config.create_if_missing {
            return Err(CaptureError::InvalidConfig(format!(
                "logical replication slot {} does not exist",
                self.config.slot_name
            )));
        }

        let row = self
            .client
            .query_one(
                "select slot_name, lsn::text from pg_create_logical_replication_slot($1, 'pgoutput')",
                &[&self.config.slot_name],
            )
            .await?;
        debug!(slot = %self.config.slot_name, "created logical replication slot");
        Ok(LogicalSlotBootstrap {
            created: true,
            consistent_lsn: row.get(1),
        })
    }

    pub async fn create_exported_logical_slot(
        config: PgCaptureConfig,
    ) -> Result<ExportedLogicalSlot> {
        config.validate()?;
        let mut connection =
            ReplicationBootstrapConnection::connect(&config.connection_uri).await?;
        let slot = connection
            .create_logical_slot_with_exported_snapshot(&config.slot_name)
            .await?;

        Ok(ExportedLogicalSlot {
            slot_name: slot.slot_name,
            consistent_lsn: slot.consistent_lsn,
            snapshot_name: slot.snapshot_name.ok_or_else(|| {
                CaptureError::ReplicationProtocol(
                    "CREATE_REPLICATION_SLOT did not return an exported snapshot name".to_string(),
                )
            })?,
            output_plugin: slot.output_plugin,
            _holder: connection,
        })
    }
}
