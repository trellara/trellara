use std::collections::HashMap;
use std::path::PathBuf;

use trellara_protocol::RelationSchemaVersion;

use crate::assembler_spill_config::StreamSpillConfig;
use crate::assembler_stream::StreamedTransactions;
use crate::assembler_transaction::OpenTransaction;

#[derive(Default)]
pub struct TransactionAssembler {
    pub(crate) open: Option<OpenTransaction>,
    pub(crate) streamed: StreamedTransactions,
    pub(crate) relation_schema_versions: HashMap<u32, RelationSchemaVersion>,
    pub(crate) stream_spill: StreamSpillConfig,
}

impl TransactionAssembler {
    pub fn with_stream_spill_threshold(streamed_spill_threshold_changes: usize) -> Self {
        Self::with_stream_spill_config(streamed_spill_threshold_changes, None)
    }

    pub fn with_stream_spill_config(
        streamed_spill_threshold_changes: usize,
        stream_spill_dir: Option<PathBuf>,
    ) -> Self {
        Self {
            stream_spill: StreamSpillConfig::new(
                streamed_spill_threshold_changes,
                stream_spill_dir,
            ),
            ..Self::default()
        }
    }
}
