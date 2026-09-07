use super::RuntimeQueueError;
use crate::{NativeRelayWireFrame, MAX_LOGICAL_FRAME_BYTES};

const MAX_IDENTITY_BYTES: usize = 255;

#[derive(Clone, Copy)]
#[repr(C)]
pub(crate) struct RuntimeQueuedFrame {
    occupied: bool,
    xid: u32,
    commit_lsn: u64,
    source_len: u16,
    dataset_len: u16,
    payload_len: u32,
    payload_digest: [u8; 32],
    source_id: [u8; MAX_IDENTITY_BYTES],
    dataset_id: [u8; MAX_IDENTITY_BYTES],
    payload: [u8; MAX_LOGICAL_FRAME_BYTES],
}

impl RuntimeQueuedFrame {
    pub(in crate::pg_shared_queue) const EMPTY: Self = Self {
        occupied: false,
        xid: 0,
        commit_lsn: 0,
        source_len: 0,
        dataset_len: 0,
        payload_len: 0,
        payload_digest: [0; 32],
        source_id: [0; MAX_IDENTITY_BYTES],
        dataset_id: [0; MAX_IDENTITY_BYTES],
        payload: [0; MAX_LOGICAL_FRAME_BYTES],
    };

    pub(in crate::pg_shared_queue) fn from_wire(
        frame: &NativeRelayWireFrame,
    ) -> Result<Self, RuntimeQueueError> {
        let mut queued = Self::EMPTY;
        queued.occupied = true;
        queued.xid = frame.xid;
        queued.commit_lsn = frame.commit_lsn;
        queued.payload_digest = frame.payload_digest;
        queued.source_len = copy_identity(&frame.source_id, &mut queued.source_id, "source_id")?;
        queued.dataset_len =
            copy_identity(&frame.dataset_id, &mut queued.dataset_id, "dataset_id")?;
        queued.payload_len = u32::try_from(frame.payload.len())
            .map_err(|_| RuntimeQueueError::PayloadTooLarge(frame.payload.len()))?;
        queued.payload[..frame.payload.len()].copy_from_slice(&frame.payload);
        Ok(queued)
    }

    pub(crate) fn to_wire(self) -> NativeRelayWireFrame {
        let source_len = usize::from(self.source_len);
        let dataset_len = usize::from(self.dataset_len);
        let payload_len = usize::try_from(self.payload_len).expect("payload length fits usize");
        NativeRelayWireFrame {
            xid: self.xid,
            commit_lsn: self.commit_lsn,
            source_id: String::from_utf8_lossy(&self.source_id[..source_len]).into_owned(),
            dataset_id: String::from_utf8_lossy(&self.dataset_id[..dataset_len]).into_owned(),
            payload: self.payload[..payload_len].to_vec(),
            payload_digest: self.payload_digest,
        }
    }

    pub(in crate::pg_shared_queue) fn matches(&self, frame: &RuntimeQueuedFrame) -> bool {
        self.occupied
            && self.xid == frame.xid
            && self.commit_lsn == frame.commit_lsn
            && self.payload_digest == frame.payload_digest
    }

    pub(crate) fn commit_lsn(&self) -> u64 {
        self.commit_lsn
    }

    pub(crate) fn payload_digest(&self) -> [u8; 32] {
        self.payload_digest
    }
}

fn copy_identity(
    value: &str,
    target: &mut [u8; MAX_IDENTITY_BYTES],
    field: &'static str,
) -> Result<u16, RuntimeQueueError> {
    if value.len() > target.len() {
        return Err(RuntimeQueueError::IdentityTooLong {
            field,
            length: value.len(),
        });
    }
    target[..value.len()].copy_from_slice(value.as_bytes());
    Ok(u16::try_from(value.len()).expect("identity length fits u16"))
}
