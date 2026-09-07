use tracing::debug;

use crate::{quote_ident, CaptureError, PgCapture, Result, TableSelector};

impl PgCapture {
    pub async fn ensure_publication(&self) -> Result<()> {
        let exists = self
            .client
            .query_opt(
                "select 1 from pg_publication where pubname = $1",
                &[&self.config.publication_name],
            )
            .await?
            .is_some();

        if exists {
            debug!(publication = %self.config.publication_name, "publication already exists");
            return Ok(());
        }

        if !self.config.create_if_missing {
            return Err(CaptureError::InvalidConfig(format!(
                "publication {} does not exist",
                self.config.publication_name
            )));
        }

        let tables = self
            .config
            .tables
            .iter()
            .map(TableSelector::to_qualified_sql)
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!(
            "create publication {} for table {}",
            quote_ident(&self.config.publication_name),
            tables
        );
        self.client.batch_execute(&sql).await?;
        debug!(publication = %self.config.publication_name, "created publication");
        Ok(())
    }
}
