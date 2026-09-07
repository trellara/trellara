use crate::{Result, TransactionKey};

pub(crate) fn validate_transaction_key(transaction: &TransactionKey) -> Result<()> {
    transaction.validate()
}
