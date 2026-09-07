use std::fs::{self, File};
use std::io::{ErrorKind, Read, Write};
use std::path::Path;

use crate::index::IndexRead;
use crate::index_entries::{encode_index_entries, parse_index_entries};
use crate::{sync_parent_dir, LocalDurability, LocalStreamError, Result};

pub(crate) fn read_index(path: &Path, log_len: u64) -> Result<IndexRead> {
    let mut bytes = Vec::new();
    match File::open(path) {
        Ok(mut file) => {
            file.read_to_end(&mut bytes)
                .map_err(|source| LocalStreamError::Io {
                    path: path.display().to_string(),
                    source,
                })?;
        }
        Err(source) if source.kind() == ErrorKind::NotFound => return Ok(IndexRead::Missing),
        Err(source) => {
            return Err(LocalStreamError::Io {
                path: path.display().to_string(),
                source,
            });
        }
    }
    Ok(parse_index_entries(&bytes, log_len))
}

pub(crate) fn write_index(
    path: &Path,
    positions: &[u64],
    durability: LocalDurability,
) -> Result<()> {
    let temp = path.with_extension("idx.tmp");
    {
        let mut file = File::create(&temp).map_err(|source| LocalStreamError::Io {
            path: temp.display().to_string(),
            source,
        })?;
        let bytes = encode_index_entries(positions)?;
        file.write_all(&bytes)
            .map_err(|source| LocalStreamError::Io {
                path: temp.display().to_string(),
                source,
            })?;
        if durability.sync_enabled() {
            file.sync_all().map_err(|source| LocalStreamError::Io {
                path: temp.display().to_string(),
                source,
            })?;
        }
    }
    fs::rename(&temp, path).map_err(|source| LocalStreamError::Io {
        path: path.display().to_string(),
        source,
    })?;
    sync_parent_dir(path, durability)
}
