use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use trellara_protocol::{Operation, RelationId, ReplicaIdentity, RowImage};

use crate::assembler_spill::StreamedChangeSpill;
use crate::Result;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct PendingChange {
    pub(crate) total_order: u32,
    pub(crate) stream_subtransaction_id: Option<String>,
    pub(crate) relation: RelationId,
    pub(crate) operation: Operation,
    pub(crate) replica_identity: ReplicaIdentity,
    pub(crate) before: Option<RowImage>,
    pub(crate) after: Option<RowImage>,
}

#[derive(Debug)]
pub(crate) enum PendingChangeBuffer {
    Memory {
        changes: Vec<PendingChange>,
        spill_threshold: usize,
        spill_dir: Option<PathBuf>,
    },
    Spilled(StreamedChangeSpill),
}

impl PendingChangeBuffer {
    pub(crate) fn memory(spill_threshold: usize) -> Self {
        Self::memory_in(spill_threshold, None)
    }

    pub(crate) fn memory_in(spill_threshold: usize, spill_dir: Option<PathBuf>) -> Self {
        Self::Memory {
            changes: Vec::new(),
            spill_threshold,
            spill_dir,
        }
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            Self::Memory { changes, .. } => changes.len(),
            Self::Spilled(spill) => spill.len,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[cfg(test)]
    pub(crate) fn is_spilled(&self) -> bool {
        matches!(self, Self::Spilled(_))
    }

    #[cfg(test)]
    pub(crate) fn spilled_path(&self) -> Option<PathBuf> {
        match self {
            Self::Spilled(spill) => Some(spill.path.clone()),
            Self::Memory { .. } => None,
        }
    }

    pub(crate) fn push(&mut self, change: PendingChange) -> Result<()> {
        match self {
            Self::Memory {
                changes,
                spill_threshold,
                ..
            } if changes.len() < *spill_threshold => {
                changes.push(change);
                Ok(())
            }
            Self::Memory {
                changes, spill_dir, ..
            } => {
                let mut spill = StreamedChangeSpill::create(spill_dir.as_ref())?;
                for existing in changes.drain(..) {
                    spill.append(&existing)?;
                }
                spill.append(&change)?;
                *self = Self::Spilled(spill);
                Ok(())
            }
            Self::Spilled(spill) => spill.append(&change),
        }
    }

    pub(crate) fn retain(&mut self, keep: impl FnMut(&PendingChange) -> bool) -> Result<()> {
        match self {
            Self::Memory { changes, .. } => {
                changes.retain(keep);
                Ok(())
            }
            Self::Spilled(spill) => spill.retain(keep),
        }
    }

    pub(crate) fn into_changes(self) -> Result<Vec<PendingChange>> {
        match self {
            Self::Memory { changes, .. } => Ok(changes),
            Self::Spilled(spill) => spill.read_all(),
        }
    }

    pub(crate) fn ordered_relation_oids(&self) -> Result<Vec<(u32, u32)>> {
        match self {
            Self::Memory { changes, .. } => Ok(changes
                .iter()
                .map(|change| (change.total_order, change.relation.oid))
                .collect::<Vec<_>>()),
            Self::Spilled(spill) => Ok(spill
                .read_all_ref()?
                .into_iter()
                .map(|change| (change.total_order, change.relation.oid))
                .collect::<Vec<_>>()),
        }
    }
}
