use trellara_protocol::{ChangeRecord, ProtocolError, TransactionBoundaryKey, TransactionEnvelope};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RawCdcTransactionKey {
    boundary: TransactionBoundaryKey,
}

impl RawCdcTransactionKey {
    pub(super) fn new(envelope: &TransactionEnvelope) -> Result<Self, ProtocolError> {
        Ok(Self {
            boundary: envelope.boundary_key()?,
        })
    }

    pub(super) fn display_identity(&self) -> String {
        self.boundary.to_string()
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct RawCdcIdempotencyKey {
    database_id: String,
    dataset_id: String,
    idempotency_key: String,
}

impl RawCdcIdempotencyKey {
    pub(super) fn new(envelope: &TransactionEnvelope, change: &ChangeRecord) -> Self {
        Self {
            database_id: envelope.database_id.clone(),
            dataset_id: envelope.dataset_id.clone(),
            idempotency_key: change.idempotency_key.clone(),
        }
    }
}

pub(super) fn transaction_identity(
    envelope: &TransactionEnvelope,
) -> Result<String, ProtocolError> {
    Ok(RawCdcTransactionKey::new(envelope)?.display_identity())
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellara_protocol::{
        idempotency_key, ChangeRecord, ColumnValue, Operation, RelationId, ReplicaIdentity,
        RowImage, StrictEnvelope,
    };

    #[test]
    fn transaction_key_scopes_by_source_database_dataset_transaction_and_commit_lsn() {
        let base = envelope("store-001", "retail", "tx-1", "0/16B6C50");
        let same_transaction_other_source = envelope("store-002", "retail", "tx-1", "0/16B6C50");
        let same_transaction_other_dataset =
            envelope("store-001", "wholesale", "tx-1", "0/16B6C50");
        let same_transaction_other_commit = envelope("store-001", "retail", "tx-1", "0/16B6D00");

        assert_ne!(
            RawCdcTransactionKey::new(&base).expect("base key"),
            RawCdcTransactionKey::new(&same_transaction_other_source).expect("source key")
        );
        assert_ne!(
            RawCdcTransactionKey::new(&base).expect("base key"),
            RawCdcTransactionKey::new(&same_transaction_other_dataset).expect("dataset key")
        );
        assert_ne!(
            RawCdcTransactionKey::new(&base).expect("base key"),
            RawCdcTransactionKey::new(&same_transaction_other_commit).expect("commit key")
        );
        assert_eq!(
            RawCdcTransactionKey::new(&base)
                .expect("base key")
                .display_identity(),
            "store-001:postgres:retail:tx-1:0/16B6C50"
        );
    }

    #[test]
    fn transaction_key_canonicalizes_commit_lsn_text() {
        let envelope = envelope("store-001", "retail", "tx-1", "00000000/016B6C50");

        assert_eq!(
            RawCdcTransactionKey::new(&envelope)
                .expect("boundary key")
                .display_identity(),
            "store-001:postgres:retail:tx-1:0/16B6C50"
        );
    }

    #[test]
    fn idempotency_key_scopes_replay_evidence_by_database_and_dataset() {
        let retail = envelope("store-001", "retail", "tx-1", "0/16B6C50");
        let mut wholesale = envelope("store-001", "wholesale", "tx-2", "0/16B6D00");
        wholesale.changes[0].idempotency_key = retail.changes[0].idempotency_key.clone();
        let mut analytics = envelope("store-001", "retail", "tx-3", "0/16B6E00");
        analytics.database_id = "analytics".to_string();
        analytics.changes[0].idempotency_key = retail.changes[0].idempotency_key.clone();

        assert_ne!(
            RawCdcIdempotencyKey::new(&retail, &retail.changes[0]),
            RawCdcIdempotencyKey::new(&wholesale, &wholesale.changes[0])
        );
        assert_ne!(
            RawCdcIdempotencyKey::new(&retail, &retail.changes[0]),
            RawCdcIdempotencyKey::new(&analytics, &analytics.changes[0])
        );
    }

    fn envelope(
        source_id: &str,
        dataset_id: &str,
        transaction_id: &str,
        commit_lsn: &str,
    ) -> TransactionEnvelope {
        let change = ChangeRecord {
            transaction_id: transaction_id.to_string(),
            total_order: 1,
            table_order: 1,
            partition_order: 1,
            relation: Some(RelationId::new(16_384, "public", "sales")),
            operation: Operation::Insert as i32,
            replica_identity: ReplicaIdentity::Default as i32,
            before: None,
            after: Some(RowImage::new(vec![ColumnValue::text(
                "id", 25, "sale-1", true,
            )])),
            idempotency_key: idempotency_key(source_id, commit_lsn, transaction_id, 1),
        };
        TransactionEnvelope::strict(StrictEnvelope {
            source_id: source_id.to_string(),
            database_id: "postgres".to_string(),
            dataset_id: dataset_id.to_string(),
            transaction_id: transaction_id.to_string(),
            begin_lsn: "0/16B6C00".to_string(),
            commit_lsn: commit_lsn.to_string(),
            commit_timestamp_ms: 1_786_420_000_000,
            changes: vec![change],
        })
    }
}
