use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::Path;

use trellara_pg_extension::NativeRelayWireFrame;

use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};

#[path = "native_spool/record_io.rs"]
mod record_io;

use record_io::{checked_message_length, read_record_length, sync_parent, truncate_partial_record};

pub(crate) const MAX_NATIVE_MESSAGE_BYTES: usize =
    trellara_pg_extension::MAX_LOGICAL_FRAME_BYTES + 2_048;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DurableFrameKey {
    source_id: String,
    dataset_id: String,
    commit_lsn: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NativeSpoolWrite {
    Appended,
    AlreadyDurable,
}

pub(crate) struct NativeSpool {
    file: File,
    durable: BTreeMap<DurableFrameKey, [u8; 32]>,
}

impl NativeSpool {
    pub(crate) fn open(path: &Path, secret: &[u8]) -> NativeRelayTransportResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let created = !path.exists();
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(path)?;
        if created {
            sync_parent(path)?;
        }
        let mut spool = Self {
            file,
            durable: BTreeMap::new(),
        };
        spool.load(secret)?;
        Ok(spool)
    }

    pub(crate) fn persist(
        &mut self,
        frame: &NativeRelayWireFrame,
        encoded: &[u8],
    ) -> NativeRelayTransportResult<NativeSpoolWrite> {
        let key = frame_key(frame);
        if let Some(digest) = self.durable.get(&key) {
            return if digest == &frame.payload_digest {
                Ok(NativeSpoolWrite::AlreadyDurable)
            } else {
                Err(NativeRelayTransportError::ConflictingDurableFrame {
                    commit_lsn: frame.commit_lsn,
                })
            };
        }
        let length = checked_message_length(encoded.len())?;
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&length.to_be_bytes())?;
        self.file.write_all(encoded)?;
        self.file.sync_all()?;
        self.durable.insert(key, frame.payload_digest);
        Ok(NativeSpoolWrite::Appended)
    }

    fn load(&mut self, secret: &[u8]) -> NativeRelayTransportResult<()> {
        self.file.seek(SeekFrom::Start(0))?;
        loop {
            let record_start = self.file.stream_position()?;
            let Some(length) = read_record_length(&mut self.file, record_start)? else {
                break;
            };
            if usize::try_from(length).unwrap_or(usize::MAX) > MAX_NATIVE_MESSAGE_BYTES {
                return Err(NativeRelayTransportError::MessageTooLarge(usize::try_from(
                    length,
                )?));
            }
            let mut encoded = vec![0; usize::try_from(length)?];
            if let Err(error) = self.file.read_exact(&mut encoded) {
                if error.kind() == ErrorKind::UnexpectedEof {
                    truncate_partial_record(&mut self.file, record_start)?;
                    break;
                }
                return Err(error.into());
            }
            let frame = NativeRelayWireFrame::decode_authenticated(&encoded, secret)?;
            let key = frame_key(&frame);
            match self.durable.insert(key, frame.payload_digest) {
                Some(digest) if digest != frame.payload_digest => {
                    return Err(NativeRelayTransportError::ConflictingDurableFrame {
                        commit_lsn: frame.commit_lsn,
                    });
                }
                _ => {}
            }
        }
        self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }
}

fn frame_key(frame: &NativeRelayWireFrame) -> DurableFrameKey {
    DurableFrameKey {
        source_id: frame.source_id.clone(),
        dataset_id: frame.dataset_id.clone(),
        commit_lsn: frame.commit_lsn,
    }
}
