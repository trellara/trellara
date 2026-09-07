use trellara_protocol::TransactionEnvelope;

use crate::Result;

pub fn strict_transaction_key(envelope: &TransactionEnvelope) -> Result<String> {
    Ok(envelope.boundary_key()?.to_string())
}

pub fn partition_chunk_key(envelope: &TransactionEnvelope, partition_id: u32) -> Result<String> {
    Ok(format!("{}:{partition_id}", envelope.boundary_key()?))
}
