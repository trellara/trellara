use std::collections::BTreeMap;
use std::fs::{self, File, OpenOptions};
use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use trellara_pg_extension::NativeRelayWireFrame;

use crate::native_publish_proof::{
    frame_key, NativeFrameKey, NativeKafkaPublishProof, NativePublishDestination,
    MAX_PUBLISH_PROOF_BYTES,
};
use crate::native_transport::{NativeRelayTransportError, NativeRelayTransportResult};

pub(crate) struct NativePublishLedger {
    file: File,
    proofs: BTreeMap<NativeFrameKey, NativeKafkaPublishProof>,
}

impl NativePublishLedger {
    pub(crate) fn open(path: &Path) -> NativeRelayTransportResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let created = !path.exists();
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .mode(0o600)
            .open(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
        if created {
            sync_parent(path)?;
        }
        let mut ledger = Self {
            file,
            proofs: BTreeMap::new(),
        };
        ledger.load()?;
        Ok(ledger)
    }

    pub(crate) fn proven(
        &self,
        frame: &NativeRelayWireFrame,
        destination: &NativePublishDestination,
    ) -> NativeRelayTransportResult<bool> {
        let Some(proof) = self.proofs.get(&frame_key(frame)) else {
            return Ok(false);
        };
        if proof.payload_digest == frame.payload_digest && proof.destination == *destination {
            Ok(true)
        } else {
            Err(NativeRelayTransportError::ConflictingKafkaProof {
                commit_lsn: frame.commit_lsn,
            })
        }
    }

    pub(crate) fn persist(
        &mut self,
        proof: NativeKafkaPublishProof,
    ) -> NativeRelayTransportResult<()> {
        if let Some(existing) = self.proofs.get(&proof.key) {
            return if existing == &proof {
                Ok(())
            } else {
                Err(NativeRelayTransportError::ConflictingKafkaProof {
                    commit_lsn: proof.key.commit_lsn,
                })
            };
        }
        let encoded = proof.encode()?;
        let length = u32::try_from(encoded.len())?;
        self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&length.to_be_bytes())?;
        self.file.write_all(&encoded)?;
        self.file.sync_all()?;
        self.proofs.insert(proof.key.clone(), proof);
        Ok(())
    }

    fn load(&mut self) -> NativeRelayTransportResult<()> {
        self.file.seek(SeekFrom::Start(0))?;
        loop {
            let start = self.file.stream_position()?;
            let Some(length) = read_length(&mut self.file, start)? else {
                break;
            };
            let length = usize::try_from(length)?;
            if length == 0 || length > MAX_PUBLISH_PROOF_BYTES {
                return Err(NativeRelayTransportError::PublishProofCorrupt(
                    "publish proof record length is outside its bound",
                ));
            }
            let mut encoded = vec![0; length];
            if let Err(error) = self.file.read_exact(&mut encoded) {
                if error.kind() == ErrorKind::UnexpectedEof {
                    truncate_tail(&mut self.file, start)?;
                    break;
                }
                return Err(error.into());
            }
            let proof = NativeKafkaPublishProof::decode(&encoded)?;
            match self.proofs.insert(proof.key.clone(), proof.clone()) {
                Some(existing) if existing != proof => {
                    return Err(NativeRelayTransportError::ConflictingKafkaProof {
                        commit_lsn: proof.key.commit_lsn,
                    });
                }
                _ => {}
            }
        }
        self.file.seek(SeekFrom::End(0))?;
        Ok(())
    }
}

fn read_length(file: &mut File, start: u64) -> NativeRelayTransportResult<Option<u32>> {
    let mut bytes = [0; 4];
    match file.read_exact(&mut bytes) {
        Ok(()) => Ok(Some(u32::from_be_bytes(bytes))),
        Err(error) if error.kind() == ErrorKind::UnexpectedEof => {
            if file.stream_position()? != start {
                truncate_tail(file, start)?;
            }
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

fn truncate_tail(file: &mut File, start: u64) -> NativeRelayTransportResult<()> {
    file.set_len(start)?;
    file.sync_all()?;
    file.seek(SeekFrom::Start(start))?;
    Ok(())
}

fn sync_parent(path: &Path) -> NativeRelayTransportResult<()> {
    if let Some(parent) = path.parent() {
        File::open(parent)?.sync_all()?;
    }
    Ok(())
}
