use tokio_postgres::Transaction;
use trellara_checkpoint::TransactionKey;
use trellara_protocol::{ProtocolError, TransactionEnvelope};

use crate::checkpoint_sql::{
    CLEAR_QUARANTINED_TRANSACTION, RECORD_APPLIED_TRANSACTION, TRANSACTION_ALREADY_APPLIED,
    UPSERT_FLOW_CHECKPOINT, UPSERT_PARTITION_CHECKPOINT,
};
use crate::{ApplyError, Result};

pub(crate) async fn transaction_already_applied(
    transaction: &Transaction<'_>,
    key: &TransactionKey,
) -> Result<bool> {
    Ok(transaction
        .query_opt(
            TRANSACTION_ALREADY_APPLIED,
            &[
                &key.source_id,
                &key.database_id,
                &key.dataset_id,
                &key.transaction_id,
                &key.commit_lsn,
            ],
        )
        .await?
        .is_some())
}

pub(crate) async fn record_applied_transaction(
    transaction: &Transaction<'_>,
    key: &TransactionKey,
) -> Result<u64> {
    Ok(transaction
        .execute(
            RECORD_APPLIED_TRANSACTION,
            &[
                &key.source_id,
                &key.database_id,
                &key.dataset_id,
                &key.transaction_id,
                &key.commit_lsn,
            ],
        )
        .await?)
}

pub(crate) async fn clear_quarantined_transaction(
    transaction: &Transaction<'_>,
    key: &TransactionKey,
) -> Result<u64> {
    Ok(transaction
        .execute(
            CLEAR_QUARANTINED_TRANSACTION,
            &[
                &key.source_id,
                &key.database_id,
                &key.dataset_id,
                &key.transaction_id,
                &key.commit_lsn,
            ],
        )
        .await?)
}

pub(crate) async fn upsert_checkpoint(
    transaction: &Transaction<'_>,
    envelope: &TransactionEnvelope,
) -> Result<u64> {
    Ok(transaction
        .execute(
            UPSERT_FLOW_CHECKPOINT,
            &[
                &envelope.source_id,
                &envelope.dataset_id,
                &envelope.commit_lsn,
            ],
        )
        .await?)
}

pub(crate) async fn upsert_partition_checkpoints(
    transaction: &Transaction<'_>,
    envelope: &TransactionEnvelope,
) -> Result<u64> {
    let Some(manifest) = &envelope.manifest else {
        return Ok(0);
    };

    let mut affected_rows = 0;
    for partition in &manifest.partitions {
        let partition_id = i32::try_from(partition.id)
            .map_err(|_| ApplyError::Protocol(ProtocolError::InvalidPartitionCount))?;
        affected_rows += transaction
            .execute(
                UPSERT_PARTITION_CHECKPOINT,
                &[
                    &envelope.source_id,
                    &envelope.dataset_id,
                    &partition_id,
                    &envelope.commit_lsn,
                ],
            )
            .await?;
    }

    Ok(affected_rows)
}
