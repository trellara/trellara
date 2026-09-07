use super::support::*;
use trellara_checkpoint::{CheckpointStore, PartitionCheckpoint};
use trellara_protocol::Checkpoint;

#[tokio::test]
async fn postgres_store_persists_flow_and_partition_checkpoints() -> TestResult<()> {
    let Some(database_url) = integration_database_url() else {
        return Ok(());
    };
    let store = connect_store(&database_url).await?;
    reset_store(&database_url).await?;

    let flow = flow_key();
    assert_eq!(store.load_checkpoint(&flow).await?, None);

    store
        .save_checkpoint(Checkpoint {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            last_seen_lsn: "0/16B6C50".to_string(),
            last_durable_lsn: "0/16B6C50".to_string(),
            last_applied_lsn: String::new(),
        })
        .await?;

    let checkpoint = store
        .load_checkpoint(&flow)
        .await?
        .expect("persisted checkpoint");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B6C50");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B6C50");
    assert_eq!(checkpoint.last_applied_lsn, "");

    store
        .save_checkpoint(Checkpoint {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            last_seen_lsn: "0/16B9000".to_string(),
            last_durable_lsn: "0/16B8800".to_string(),
            last_applied_lsn: "0/16B8000".to_string(),
        })
        .await?;
    store
        .save_checkpoint(Checkpoint {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            last_seen_lsn: String::new(),
            last_durable_lsn: String::new(),
            last_applied_lsn: String::new(),
        })
        .await?;

    let checkpoint = store
        .load_checkpoint(&flow)
        .await?
        .expect("checkpoint after empty merge");
    assert_eq!(checkpoint.last_seen_lsn, "0/16B9000");
    assert_eq!(checkpoint.last_durable_lsn, "0/16B8800");
    assert_eq!(checkpoint.last_applied_lsn, "0/16B8000");

    store
        .record_partition_checkpoint(PartitionCheckpoint {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            partition_id: 2,
            last_durable_lsn: "0/16B9000".to_string(),
            last_applied_lsn: "0/16B8800".to_string(),
        })
        .await?;
    store
        .record_partition_checkpoint(PartitionCheckpoint {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            partition_id: 2,
            last_durable_lsn: "0/16B7000".to_string(),
            last_applied_lsn: String::new(),
        })
        .await?;
    let partition_checkpoints = store.load_partition_checkpoints(&flow).await?;
    assert_eq!(
        partition_checkpoints,
        vec![PartitionCheckpoint {
            source_id: SOURCE_ID.to_string(),
            dataset_id: DATASET_ID.to_string(),
            partition_id: 2,
            last_durable_lsn: "0/16B9000".to_string(),
            last_applied_lsn: "0/16B8800".to_string(),
        }]
    );

    Ok(())
}
