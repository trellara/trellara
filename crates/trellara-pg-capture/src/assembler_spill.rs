use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::assembler_buffer::PendingChange;
use crate::{CaptureError, Result};

static STREAM_SPILL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
pub(crate) struct StreamedChangeSpill {
    pub(crate) path: PathBuf,
    pub(crate) len: usize,
}

impl StreamedChangeSpill {
    pub(crate) fn create(spill_dir: Option<&PathBuf>) -> Result<Self> {
        let path = unique_spill_path(spill_dir);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        File::create(&path)?;
        Ok(Self { path, len: 0 })
    }

    pub(crate) fn append(&mut self, change: &PendingChange) -> Result<()> {
        let mut file = OpenOptions::new().append(true).open(&self.path)?;
        serde_json::to_writer(&mut file, change)?;
        file.write_all(b"\n")?;
        file.sync_data()?;
        self.len += 1;
        Ok(())
    }

    pub(crate) fn read_all(self) -> Result<Vec<PendingChange>> {
        read_spilled_changes(&self.path)
    }

    pub(crate) fn read_all_ref(&self) -> Result<Vec<PendingChange>> {
        read_spilled_changes(&self.path)
    }

    pub(crate) fn retain(&mut self, mut keep: impl FnMut(&PendingChange) -> bool) -> Result<()> {
        let retained = read_spilled_changes(&self.path)?
            .into_iter()
            .filter(|change| keep(change))
            .collect::<Vec<_>>();
        let temp = self.path.with_extension("tmp");
        {
            let mut file = File::create(&temp)?;
            for change in &retained {
                serde_json::to_writer(&mut file, change)?;
                file.write_all(b"\n")?;
            }
            file.sync_all()?;
        }
        fs::rename(&temp, &self.path)?;
        self.len = retained.len();
        Ok(())
    }
}

impl Drop for StreamedChangeSpill {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn read_spilled_changes(path: &Path) -> Result<Vec<PendingChange>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut changes = Vec::new();
    for (index, line) in reader.lines().enumerate() {
        let line = line?;
        changes.push(serde_json::from_str(&line).map_err(|source| {
            CaptureError::StreamSpillCorrupt {
                path: path.display().to_string(),
                line: index + 1,
                source,
            }
        })?);
    }
    Ok(changes)
}

fn unique_spill_path(spill_dir: Option<&PathBuf>) -> PathBuf {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = STREAM_SPILL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    spill_dir
        .cloned()
        .unwrap_or_else(std::env::temp_dir)
        .join(format!(
            "trellara-pg-capture-stream-{}-{now}-{sequence}.jsonl",
            std::process::id()
        ))
}
