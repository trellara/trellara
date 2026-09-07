use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use apache_iceberg::memory::{MemoryCatalogBuilder, MEMORY_CATALOG_WAREHOUSE};
use apache_iceberg::{
    Catalog, CatalogBuilder, Namespace, NamespaceIdent, TableCommit, TableCreation, TableIdent,
};
use async_trait::async_trait;
use trellara_lake::{raw_cdc_lake_ddl_ack_evidence, RawCdcLakeDdlAckRequest};

use super::*;

#[tokio::test]
async fn concurrent_writers_converge_on_one_snapshot() {
    let catalog = Arc::new(memory_catalog().await);
    let plan = provisioned_commit_plan(catalog.as_ref()).await;
    let table = plan.tables[0].clone();
    let mut tasks = Vec::new();
    for _ in 0..16 {
        let catalog = catalog.clone();
        let table = table.clone();
        tasks.push(tokio::spawn(async move {
            commit_iceberg_table_append(catalog.as_ref(), &table).await
        }));
    }

    let mut snapshot_ids = Vec::new();
    let mut committed = 0;
    let mut already_committed = 0;
    for task in tasks {
        let receipt = task.await.expect("writer task").expect("writer receipt");
        snapshot_ids.push(receipt.snapshot_id);
        match receipt.status {
            IcebergTableCommitStatus::Committed => committed += 1,
            IcebergTableCommitStatus::AlreadyCommitted => already_committed += 1,
        }
    }
    assert_eq!(committed, 1);
    assert_eq!(already_committed, 15);
    assert!(snapshot_ids
        .iter()
        .all(|snapshot_id| *snapshot_id == snapshot_ids[0]));
}

#[tokio::test]
async fn commit_reconciles_response_loss_after_catalog_durability() {
    let catalog = Arc::new(memory_catalog().await);
    let plan = provisioned_commit_plan(catalog.as_ref()).await;
    let catalog = LoseFirstUpdateResponseCatalog {
        inner: catalog,
        lose_response: AtomicBool::new(true),
    };

    let receipt = commit_iceberg_table_append(&catalog, &plan.tables[0])
        .await
        .expect("ambiguous commit reconciles");
    assert_eq!(receipt.status, IcebergTableCommitStatus::AlreadyCommitted);
    assert!(receipt.snapshot_id > 0);
}

async fn memory_catalog() -> apache_iceberg::MemoryCatalog {
    MemoryCatalogBuilder::default()
        .load(
            "test",
            HashMap::from([(
                MEMORY_CATALOG_WAREHOUSE.to_string(),
                format!("memory:///warehouse-{}", uuid::Uuid::new_v4()),
            )]),
        )
        .await
        .expect("memory catalog")
}

async fn provisioned_commit_plan(catalog: &dyn Catalog) -> IcebergEpochCommitPlan {
    let write_plan = raw_cdc_plan();
    let config = commit_config();
    let provisioning =
        plan_raw_cdc_iceberg_table_provisioning(&write_plan, &config).expect("provisioning plan");
    let evidence = raw_cdc_lake_ddl_ack_evidence(RawCdcLakeDdlAckRequest {
        source_id: "source-a".to_string(),
        database_id: "database-a".to_string(),
        dataset_id: "retail".to_string(),
        barrier_id: "barrier-1".to_string(),
        ack_lsn: "0/16B6C50".to_string(),
        schema_version: "schema-v2".to_string(),
        epoch_id: write_plan.epoch_id.clone(),
        metadata_table: write_plan.epoch_metadata.epochs_table.clone(),
        partition_metadata_table: write_plan.epoch_metadata.epoch_partitions_table.clone(),
        manifest_digest: write_plan.epoch_metadata.epoch_row.manifest_digest.clone(),
    })
    .expect("lake ack");
    let ack = IcebergDdlAcknowledgement::from_raw_cdc_lake_ack(
        &evidence,
        provisioning
            .iter()
            .map(|plan| plan.schema_fingerprint_sha256.clone()),
    )
    .expect("DDL authorization");
    provision_raw_cdc_iceberg_tables(catalog, "retail", &provisioning, Some(&ack))
        .await
        .expect("provision tables");
    plan_iceberg_epoch_commit(&write_plan, &config, completed_files()).expect("commit plan")
}

#[derive(Debug)]
struct LoseFirstUpdateResponseCatalog {
    inner: Arc<apache_iceberg::MemoryCatalog>,
    lose_response: AtomicBool,
}

#[async_trait]
impl Catalog for LoseFirstUpdateResponseCatalog {
    async fn list_namespaces(
        &self,
        parent: Option<&NamespaceIdent>,
    ) -> apache_iceberg::Result<Vec<NamespaceIdent>> {
        self.inner.list_namespaces(parent).await
    }

    async fn create_namespace(
        &self,
        namespace: &NamespaceIdent,
        properties: HashMap<String, String>,
    ) -> apache_iceberg::Result<Namespace> {
        self.inner.create_namespace(namespace, properties).await
    }

    async fn get_namespace(&self, namespace: &NamespaceIdent) -> apache_iceberg::Result<Namespace> {
        self.inner.get_namespace(namespace).await
    }

    async fn namespace_exists(&self, namespace: &NamespaceIdent) -> apache_iceberg::Result<bool> {
        self.inner.namespace_exists(namespace).await
    }

    async fn update_namespace(
        &self,
        namespace: &NamespaceIdent,
        properties: HashMap<String, String>,
    ) -> apache_iceberg::Result<()> {
        self.inner.update_namespace(namespace, properties).await
    }

    async fn drop_namespace(&self, namespace: &NamespaceIdent) -> apache_iceberg::Result<()> {
        self.inner.drop_namespace(namespace).await
    }

    async fn list_tables(
        &self,
        namespace: &NamespaceIdent,
    ) -> apache_iceberg::Result<Vec<TableIdent>> {
        self.inner.list_tables(namespace).await
    }

    async fn create_table(
        &self,
        namespace: &NamespaceIdent,
        creation: TableCreation,
    ) -> apache_iceberg::Result<apache_iceberg::table::Table> {
        self.inner.create_table(namespace, creation).await
    }

    async fn load_table(
        &self,
        table: &TableIdent,
    ) -> apache_iceberg::Result<apache_iceberg::table::Table> {
        self.inner.load_table(table).await
    }

    async fn drop_table(&self, table: &TableIdent) -> apache_iceberg::Result<()> {
        self.inner.drop_table(table).await
    }

    async fn purge_table(&self, table: &TableIdent) -> apache_iceberg::Result<()> {
        self.inner.purge_table(table).await
    }

    async fn table_exists(&self, table: &TableIdent) -> apache_iceberg::Result<bool> {
        self.inner.table_exists(table).await
    }

    async fn rename_table(
        &self,
        source: &TableIdent,
        destination: &TableIdent,
    ) -> apache_iceberg::Result<()> {
        self.inner.rename_table(source, destination).await
    }

    async fn register_table(
        &self,
        table: &TableIdent,
        metadata_location: String,
    ) -> apache_iceberg::Result<apache_iceberg::table::Table> {
        self.inner.register_table(table, metadata_location).await
    }

    async fn update_table(
        &self,
        commit: TableCommit,
    ) -> apache_iceberg::Result<apache_iceberg::table::Table> {
        let table = self.inner.update_table(commit).await?;
        if self.lose_response.swap(false, Ordering::SeqCst) {
            Err(apache_iceberg::Error::new(
                apache_iceberg::ErrorKind::Unexpected,
                "simulated lost REST response after catalog durability",
            ))
        } else {
            Ok(table)
        }
    }
}
